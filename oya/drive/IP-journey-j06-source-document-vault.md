---
doc_class: Implementation-Plan
ip_id: IP-journey-j06-source-document-vault
journey_id: j06-press-source-securedrop-class
microservice: drive
role: source-document-vault
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

# IP - j06 - drive - source-document-vault

Goal: implement the drive portion of SecureDrop-class press source submission so A journalist source submits documents to a publisher tenant through a SecureDrop-class Community path.
Binding ADR: ADR-0300. This IP is one independently reviewable slice and must not weaken the common critical-path ADR pack.

## Scope

In scope: source-document-vault, JSON Schema validation, Cedar authorization, audit-chain emission, observability, failure branches, and integration tests for j06.
Out of scope: changing ADRs, changing standards, editing existing PRDs, bypassing Cedar, or adding a new microservice directory.

## Data model

| Object | Storage | Schema | Retention |
|---|---|---|---|
| securedrop-submission | drive.source-document-vault table or event stream | docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json | pack-controlled, minimum audit retention |
| source-document-envelope | drive.source-document-vault table or event stream | docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json | pack-controlled, minimum audit retention |
| blind-reply-token | drive.source-document-vault table or event stream | docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json | pack-controlled, minimum audit retention |

## API and event contract

```yaml
openapi: 3.2.0
info:
  title: drive j06 source-document-vault
  version: 1.0.0
paths:
  /journeys/j06/drive/source-document-vault:
    post:
      operationId: j06DriveSourceDocumentVault
      x-binding-adr: ADR-0300
      x-audit-required: true
      responses:
        "202":
          description: Accepted and audit-sealed
```

```yaml
asyncapi: 3.1.0
info:
  title: drive j06 events
  version: 1.0.0
channels:
  j06.drive.source-document-vault.accepted:
    address: j06.drive.source-document-vault.accepted
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

### Step 01 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.1.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 02 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.2.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 03 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.3.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 04 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.4.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 05 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.5.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 06 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.6.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 07 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.7.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 08 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.8.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 09 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.9.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 10 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.10.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 11 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.11.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 12 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.12.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 13 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.13.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 14 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.14.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 15 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.15.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 16 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.16.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 17 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.17.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 18 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.18.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 19 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.19.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 20 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.20.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 21 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.21.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 22 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.22.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 23 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.23.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 24 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.24.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 25 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.25.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 26 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.26.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 27 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.27.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 28 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.28.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 29 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.29.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 30 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.30.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 31 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.31.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 32 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.32.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 33 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.33.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 34 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.34.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 35 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.35.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 36 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.36.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 37 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.37.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 38 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.38.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 39 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.39.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 40 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.40.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 41 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.41.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 42 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.42.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 43 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.43.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 44 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.44.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 45 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.45.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 46 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.46.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 47 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.47.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 48 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.48.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 49 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.49.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 50 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.50.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 51 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.51.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 52 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.52.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 53 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.53.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 54 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.54.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 55 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.55.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 56 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.56.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 57 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.57.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 58 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.58.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 59 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.59.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 60 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.60.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 61 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.61.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 62 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.62.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 63 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.63.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 64 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.64.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 65 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.65.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 66 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.66.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 67 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.67.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 68 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.68.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 69 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.69.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 70 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for securedrop-submission without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.70.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 71 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for source-document-envelope without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.71.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 72 - drive source-document-vault slice detail
- Build: add or wire the source-document-vault handler for blind-reply-token without changing unrelated drive surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for drive.
- Emit: publish j06.drive.source-document-vault.accepted and seal audit class j06.drive.72.sealed.
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

- Gate 1: schema-parse passes for drive source-document-vault and stores evidence with journey_id=j06.
- Gate 2: cedar-permit-deny-forbid passes for drive source-document-vault and stores evidence with journey_id=j06.
- Gate 3: audit-seal passes for drive source-document-vault and stores evidence with journey_id=j06.
- Gate 4: trace-cardinality passes for drive source-document-vault and stores evidence with journey_id=j06.
- Gate 5: 10x-load passes for drive source-document-vault and stores evidence with journey_id=j06.
- Gate 6: replay-idempotency passes for drive source-document-vault and stores evidence with journey_id=j06.
- Gate 7: cross-tenant-negative passes for drive source-document-vault and stores evidence with journey_id=j06.
- Gate 8: pack-overlay passes for drive source-document-vault and stores evidence with journey_id=j06.
- Gate 9: operator-review passes for drive source-document-vault and stores evidence with journey_id=j06.
- Gate 10: docs-link-resolves passes for drive source-document-vault and stores evidence with journey_id=j06.
