---
doc_class: Implementation-Plan
ip_id: IP-journey-j17-metadata-minimized-dm
journey_id: j17-activist-dissident-high-risk-mode
microservice: messenger
role: metadata-minimized-dm
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
  - docs/user-journeys/j17-activist-dissident-high-risk-mode/README.md
  - docs/user-journeys/j17-activist-dissident-high-risk-mode/handshake.md
  - docs/user-journeys/j17-activist-dissident-high-risk-mode/integration-test-plan.md
---

# IP - j17 - messenger - metadata-minimized-dm

Goal: implement the messenger portion of Activist dissident high-risk mode so An activist enables HIGH_RISK_USER overlay with Tor ingress and metadata minimization.
Binding ADR: ADR-0300. This IP is one independently reviewable slice and must not weaken the common critical-path ADR pack.

## Scope

In scope: metadata-minimized-dm, JSON Schema validation, Cedar authorization, audit-chain emission, observability, failure branches, and integration tests for j17.
Out of scope: changing ADRs, changing standards, editing existing PRDs, bypassing Cedar, or adding a new microservice directory.

## Data model

| Object | Storage | Schema | Retention |
|---|---|---|---|
| high-risk-mode-activation | messenger.metadata-minimized-dm table or event stream | docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json | pack-controlled, minimum audit retention |
| metadata-minimization-policy | messenger.metadata-minimized-dm table or event stream | docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json | pack-controlled, minimum audit retention |
| tor-ingress-session | messenger.metadata-minimized-dm table or event stream | docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json | pack-controlled, minimum audit retention |

## API and event contract

```yaml
openapi: 3.2.0
info:
  title: messenger j17 metadata-minimized-dm
  version: 1.0.0
paths:
  /journeys/j17/messenger/metadata-minimized-dm:
    post:
      operationId: j17MessengerMetadataMinimizedDm
      x-binding-adr: ADR-0300
      x-audit-required: true
      responses:
        "202":
          description: Accepted and audit-sealed
```

```yaml
asyncapi: 3.1.0
info:
  title: messenger j17 events
  version: 1.0.0
channels:
  j17.messenger.metadata-minimized-dm.accepted:
    address: j17.messenger.metadata-minimized-dm.accepted
```

## Cedar and policy grammar

The caller-side policy library evaluates before network dispatch where possible. Service-side policy re-evaluates on mutation.

```cedar
permit(principal, action == Action::"j17.execute", resource)
when {
  principal.tenant_id == resource.tenant_id &&
  resource.binding_adr == "ADR-0300" &&
  context.cell_tier >= resource.minimum_cell_tier &&
  context.audit_chain_required == true
};

forbid(principal, action == Action::"j17.execute", resource)
when {
  principal.tenant_id != resource.tenant_id &&
  context.cross_tenant_grant_absent == true
};
```

## Implementation steps

### Step 01 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.1.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 02 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.2.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 03 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.3.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 04 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.4.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 05 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.5.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 06 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.6.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 07 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.7.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 08 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.8.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 09 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.9.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 10 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.10.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 11 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.11.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 12 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.12.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 13 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.13.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 14 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.14.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 15 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.15.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 16 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.16.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 17 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.17.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 18 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.18.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 19 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.19.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 20 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.20.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 21 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.21.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 22 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.22.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 23 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.23.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 24 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.24.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 25 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.25.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 26 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.26.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 27 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.27.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 28 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.28.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 29 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.29.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 30 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.30.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 31 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.31.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 32 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.32.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 33 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.33.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 34 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.34.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 35 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.35.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 36 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.36.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 37 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.37.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 38 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.38.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 39 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.39.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 40 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.40.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 41 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.41.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 42 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.42.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 43 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.43.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 44 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.44.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 45 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.45.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 46 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.46.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 47 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.47.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 48 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.48.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 49 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.49.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 50 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.50.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 51 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.51.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 52 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.52.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 53 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.53.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 54 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.54.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 55 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.55.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 56 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.56.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 57 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.57.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 58 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.58.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 59 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.59.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 60 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.60.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 61 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.61.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 62 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.62.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 63 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.63.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 64 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.64.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 65 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.65.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 66 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.66.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 67 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.67.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 68 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.68.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 69 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.69.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 70 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for high-risk-mode-activation without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/high-risk-mode-activation.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.70.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 71 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for metadata-minimization-policy without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/metadata-minimization-policy.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.71.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 72 - messenger metadata-minimized-dm slice detail
- Build: add or wire the metadata-minimized-dm handler for tor-ingress-session without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j17-activist-dissident-high-risk-mode/schemas/tor-ingress-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j17.messenger.metadata-minimized-dm.accepted and seal audit class j17.messenger.72.sealed.
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
For j17, the baseline planning rate is 100 journey starts per minute per active cell unless a stricter emergency or compliance pack overrides it.
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
- j17.journey.accepted: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j17.cedar.decision: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j17.audit.sealed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j17.handoff.completed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j17.exception.reviewed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.

Metrics emitted:
- oyatie_j17_accepted_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j17_refused_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j17_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j17_audit_seal_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j17_manual_review_queue_depth: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j17_notification_delivery_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.

Trace shape:
- span 1: identity.high-risk-user-overlay uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 2: messenger.metadata-minimized-dm uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 3: drive.encrypted-evidence-locker uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 4: community.tor-friendly-anonymous-presence uses parent trace from the journey accept span and records Cedar decision plus schema version.

## Acceptance gates

- Gate 1: schema-parse passes for messenger metadata-minimized-dm and stores evidence with journey_id=j17.
- Gate 2: cedar-permit-deny-forbid passes for messenger metadata-minimized-dm and stores evidence with journey_id=j17.
- Gate 3: audit-seal passes for messenger metadata-minimized-dm and stores evidence with journey_id=j17.
- Gate 4: trace-cardinality passes for messenger metadata-minimized-dm and stores evidence with journey_id=j17.
- Gate 5: 10x-load passes for messenger metadata-minimized-dm and stores evidence with journey_id=j17.
- Gate 6: replay-idempotency passes for messenger metadata-minimized-dm and stores evidence with journey_id=j17.
- Gate 7: cross-tenant-negative passes for messenger metadata-minimized-dm and stores evidence with journey_id=j17.
- Gate 8: pack-overlay passes for messenger metadata-minimized-dm and stores evidence with journey_id=j17.
- Gate 9: operator-review passes for messenger metadata-minimized-dm and stores evidence with journey_id=j17.
- Gate 10: docs-link-resolves passes for messenger metadata-minimized-dm and stores evidence with journey_id=j17.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/messenger/contracts/openapi/messenger.yaml`, `microservices/messenger/contracts/asyncapi/messenger-events.yaml`, `microservices/messenger/contracts/proto/messenger.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/messenger/IP-journey-j17-metadata-minimized-dm.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/messenger/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
