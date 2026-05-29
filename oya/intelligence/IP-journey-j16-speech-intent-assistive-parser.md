---
doc_class: Implementation-Plan
ip_id: IP-journey-j16-speech-intent-assistive-parser
journey_id: j16-disability-accommodation-voice-only-signup
microservice: intelligence
role: speech-intent-assistive-parser
status: draft
date: 2026-05-20
related_adrs:
  - ADR-0303
  - ADR-0298
  - ADR-0299
  - ADR-0300
  - ADR-0301
  - ADR-0302
  - ADR-0304
  - ADR-0305
  - ADR-0306
  - ADR-0292
related_journey_artifacts:
  - docs/user-journeys/j16-disability-accommodation-voice-only-signup/README.md
  - docs/user-journeys/j16-disability-accommodation-voice-only-signup/handshake.md
  - docs/user-journeys/j16-disability-accommodation-voice-only-signup/integration-test-plan.md
---

# IP - j16 - intelligence - speech-intent-assistive-parser

Goal: implement the intelligence portion of Voice-only disability accommodation signup so A post-stroke user signs up with voice-only interaction plus single-switch alternatives.
Binding ADR: ADR-0303. This IP is one independently reviewable slice and must not weaken the common critical-path ADR pack.

## Scope

In scope: speech-intent-assistive-parser, JSON Schema validation, Cedar authorization, audit-chain emission, observability, failure branches, and integration tests for j16.
Out of scope: changing ADRs, changing standards, editing existing PRDs, bypassing Cedar, or adding a new microservice directory.

## Data model

| Object | Storage | Schema | Retention |
|---|---|---|---|
| voice-only-signup-session | intelligence.speech-intent-assistive-parser table or event stream | docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json | pack-controlled, minimum audit retention |
| assistive-auth-decision | intelligence.speech-intent-assistive-parser table or event stream | docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json | pack-controlled, minimum audit retention |
| single-switch-fallback | intelligence.speech-intent-assistive-parser table or event stream | docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json | pack-controlled, minimum audit retention |

## API and event contract

```yaml
openapi: 3.2.0
info:
  title: intelligence j16 speech-intent-assistive-parser
  version: 1.0.0
paths:
  /journeys/j16/intelligence/speech-intent-assistive-parser:
    post:
      operationId: j16IntelligenceSpeechIntentAssistiveParser
      x-binding-adr: ADR-0303
      x-audit-required: true
      responses:
        "202":
          description: Accepted and audit-sealed
```

```yaml
asyncapi: 3.1.0
info:
  title: intelligence j16 events
  version: 1.0.0
channels:
  j16.intelligence.speech-intent-assistive-parser.accepted:
    address: j16.intelligence.speech-intent-assistive-parser.accepted
```

## Cedar and policy grammar

The caller-side policy library evaluates before network dispatch where possible. Service-side policy re-evaluates on mutation.

```cedar
permit(principal, action == Action::"j16.execute", resource)
when {
  principal.tenant_id == resource.tenant_id &&
  resource.binding_adr == "ADR-0303" &&
  context.cell_tier >= resource.minimum_cell_tier &&
  context.audit_chain_required == true
};

forbid(principal, action == Action::"j16.execute", resource)
when {
  principal.tenant_id != resource.tenant_id &&
  context.cross_tenant_grant_absent == true
};
```

## Implementation steps

### Step 01 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.1.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 02 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.2.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 03 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.3.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 04 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.4.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 05 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.5.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 06 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.6.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 07 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.7.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 08 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.8.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 09 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.9.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 10 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.10.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 11 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.11.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 12 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.12.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 13 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.13.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 14 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.14.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 15 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.15.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 16 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.16.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 17 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.17.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 18 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.18.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 19 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.19.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 20 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.20.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 21 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.21.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 22 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.22.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 23 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.23.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 24 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.24.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 25 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.25.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 26 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.26.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 27 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.27.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 28 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.28.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 29 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.29.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 30 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.30.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 31 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.31.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 32 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.32.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 33 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.33.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 34 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.34.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 35 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.35.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 36 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.36.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 37 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.37.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 38 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.38.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 39 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.39.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 40 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.40.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 41 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.41.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 42 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.42.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 43 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.43.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 44 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.44.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 45 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.45.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 46 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.46.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 47 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.47.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 48 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.48.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 49 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.49.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 50 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.50.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 51 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.51.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 52 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.52.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 53 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.53.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 54 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.54.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 55 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.55.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 56 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.56.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 57 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.57.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 58 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.58.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 59 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.59.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 60 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.60.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 61 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.61.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 62 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.62.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 63 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.63.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 64 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.64.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 65 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.65.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 66 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.66.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 67 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.67.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 68 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.68.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 69 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.69.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 70 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for voice-only-signup-session without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/voice-only-signup-session.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.70.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 71 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for assistive-auth-decision without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/assistive-auth-decision.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.71.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 72 - intelligence speech-intent-assistive-parser slice detail
- Build: add or wire the speech-intent-assistive-parser handler for single-switch-fallback without changing unrelated intelligence surfaces.
- Validate: parse docs/user-journeys/j16-disability-accommodation-voice-only-signup/schemas/single-switch-fallback.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0303, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for intelligence.
- Emit: publish j16.intelligence.speech-intent-assistive-parser.accepted and seal audit class j16.intelligence.72.sealed.
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
For j16, the baseline planning rate is 100 journey starts per minute per active cell unless a stricter emergency or compliance pack overrides it.
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
- j16.journey.accepted: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j16.cedar.decision: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j16.audit.sealed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j16.handoff.completed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j16.exception.reviewed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.

Metrics emitted:
- oyatie_j16_accepted_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j16_refused_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j16_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j16_audit_seal_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j16_manual_review_queue_depth: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j16_notification_delivery_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.

Trace shape:
- span 1: identity.voice-biometric-and-passkey-alternative uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 2: intelligence.speech-intent-assistive-parser uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 3: application.a11y-substrate-signup-shell uses parent trace from the journey accept span and records Cedar decision plus schema version.

## Acceptance gates

- Gate 1: schema-parse passes for intelligence speech-intent-assistive-parser and stores evidence with journey_id=j16.
- Gate 2: cedar-permit-deny-forbid passes for intelligence speech-intent-assistive-parser and stores evidence with journey_id=j16.
- Gate 3: audit-seal passes for intelligence speech-intent-assistive-parser and stores evidence with journey_id=j16.
- Gate 4: trace-cardinality passes for intelligence speech-intent-assistive-parser and stores evidence with journey_id=j16.
- Gate 5: 10x-load passes for intelligence speech-intent-assistive-parser and stores evidence with journey_id=j16.
- Gate 6: replay-idempotency passes for intelligence speech-intent-assistive-parser and stores evidence with journey_id=j16.
- Gate 7: cross-tenant-negative passes for intelligence speech-intent-assistive-parser and stores evidence with journey_id=j16.
- Gate 8: pack-overlay passes for intelligence speech-intent-assistive-parser and stores evidence with journey_id=j16.
- Gate 9: operator-review passes for intelligence speech-intent-assistive-parser and stores evidence with journey_id=j16.
- Gate 10: docs-link-resolves passes for intelligence speech-intent-assistive-parser and stores evidence with journey_id=j16.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-journey-j16-speech-intent-assistive-parser.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
