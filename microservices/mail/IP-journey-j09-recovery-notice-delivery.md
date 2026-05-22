---
doc_class: Implementation-Plan
ip_id: IP-journey-j09-recovery-notice-delivery
journey_id: j09-account-recovery-phishing-resistant
microservice: mail
role: recovery-notice-delivery
status: draft
date: 2026-05-20
related_adrs:
  - ADR-0299
  - ADR-0298
  - ADR-0300
  - ADR-0301
  - ADR-0302
  - ADR-0303
  - ADR-0304
  - ADR-0305
  - ADR-0306
  - ADR-0292
related_journey_artifacts:
  - docs/user-journeys/j09-account-recovery-phishing-resistant/README.md
  - docs/user-journeys/j09-account-recovery-phishing-resistant/handshake.md
  - docs/user-journeys/j09-account-recovery-phishing-resistant/integration-test-plan.md
---

# IP - j09 - mail - recovery-notice-delivery

Goal: implement the mail portion of Phishing-resistant account recovery so Yejin loses her phone and recovers with passkey backup, recovery code, and delegated trusted contact.
Binding ADR: ADR-0299. This IP is one independently reviewable slice and must not weaken the common critical-path ADR pack.

## Scope

In scope: recovery-notice-delivery, JSON Schema validation, Cedar authorization, audit-chain emission, observability, failure branches, and integration tests for j09.
Out of scope: changing ADRs, changing standards, editing existing PRDs, bypassing Cedar, or adding a new microservice directory.

## Data model

| Object | Storage | Schema | Retention |
|---|---|---|---|
| recovery-claim | mail.recovery-notice-delivery table or event stream | docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json | pack-controlled, minimum audit retention |
| trusted-contact-challenge | mail.recovery-notice-delivery table or event stream | docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json | pack-controlled, minimum audit retention |
| device-rebind-result | mail.recovery-notice-delivery table or event stream | docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json | pack-controlled, minimum audit retention |

## API and event contract

```yaml
openapi: 3.2.0
info:
  title: mail j09 recovery-notice-delivery
  version: 1.0.0
paths:
  /journeys/j09/mail/recovery-notice-delivery:
    post:
      operationId: j09MailRecoveryNoticeDelivery
      x-binding-adr: ADR-0299
      x-audit-required: true
      responses:
        "202":
          description: Accepted and audit-sealed
```

```yaml
asyncapi: 3.1.0
info:
  title: mail j09 events
  version: 1.0.0
channels:
  j09.mail.recovery-notice-delivery.accepted:
    address: j09.mail.recovery-notice-delivery.accepted
```

## Cedar and policy grammar

The caller-side policy library evaluates before network dispatch where possible. Service-side policy re-evaluates on mutation.

```cedar
permit(principal, action == Action::"j09.execute", resource)
when {
  principal.tenant_id == resource.tenant_id &&
  resource.binding_adr == "ADR-0299" &&
  context.cell_tier >= resource.minimum_cell_tier &&
  context.audit_chain_required == true
};

forbid(principal, action == Action::"j09.execute", resource)
when {
  principal.tenant_id != resource.tenant_id &&
  context.cross_tenant_grant_absent == true
};
```

## Implementation steps

### Step 01 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.1.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 02 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.2.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 03 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.3.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 04 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.4.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 05 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.5.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 06 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.6.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 07 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.7.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 08 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.8.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 09 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.9.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 10 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.10.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 11 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.11.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 12 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.12.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 13 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.13.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 14 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.14.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 15 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.15.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 16 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.16.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 17 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.17.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 18 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.18.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 19 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.19.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 20 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.20.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 21 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.21.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 22 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.22.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 23 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.23.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 24 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.24.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 25 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.25.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 26 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.26.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 27 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.27.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 28 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.28.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 29 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.29.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 30 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.30.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 31 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.31.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 32 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.32.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 33 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.33.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 34 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.34.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 35 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.35.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 36 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.36.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 37 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.37.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 38 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.38.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 39 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.39.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 40 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.40.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 41 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.41.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 42 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.42.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 43 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.43.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 44 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.44.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 45 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.45.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 46 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.46.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 47 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.47.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 48 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.48.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 49 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.49.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 50 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.50.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 51 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.51.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 52 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.52.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 53 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.53.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 54 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.54.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 55 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.55.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 56 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.56.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 57 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.57.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 58 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.58.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 59 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.59.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 60 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.60.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 61 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.61.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 62 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.62.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 63 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.63.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 64 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.64.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 65 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.65.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 66 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.66.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 67 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.67.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 68 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.68.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 69 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.69.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 70 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for recovery-claim without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/recovery-claim.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.70.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 71 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for trusted-contact-challenge without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/trusted-contact-challenge.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.71.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 72 - mail recovery-notice-delivery slice detail
- Build: add or wire the recovery-notice-delivery handler for device-rebind-result without changing unrelated mail surfaces.
- Validate: parse docs/user-journeys/j09-account-recovery-phishing-resistant/schemas/device-rebind-result.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0299, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for mail.
- Emit: publish j09.mail.recovery-notice-delivery.accepted and seal audit class j09.mail.72.sealed.
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
For j09, the baseline planning rate is 100 journey starts per minute per active cell unless a stricter emergency or compliance pack overrides it.
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
- j09.journey.accepted: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j09.cedar.decision: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j09.audit.sealed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j09.handoff.completed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j09.exception.reviewed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.

Metrics emitted:
- oyatie_j09_accepted_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j09_refused_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j09_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j09_audit_seal_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j09_manual_review_queue_depth: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j09_notification_delivery_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.

Trace shape:
- span 1: identity.phishing-resistant-recovery uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 2: messenger.trusted-contact-challenge uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 3: mail.recovery-notice-delivery uses parent trace from the journey accept span and records Cedar decision plus schema version.

## Acceptance gates

- Gate 1: schema-parse passes for mail recovery-notice-delivery and stores evidence with journey_id=j09.
- Gate 2: cedar-permit-deny-forbid passes for mail recovery-notice-delivery and stores evidence with journey_id=j09.
- Gate 3: audit-seal passes for mail recovery-notice-delivery and stores evidence with journey_id=j09.
- Gate 4: trace-cardinality passes for mail recovery-notice-delivery and stores evidence with journey_id=j09.
- Gate 5: 10x-load passes for mail recovery-notice-delivery and stores evidence with journey_id=j09.
- Gate 6: replay-idempotency passes for mail recovery-notice-delivery and stores evidence with journey_id=j09.
- Gate 7: cross-tenant-negative passes for mail recovery-notice-delivery and stores evidence with journey_id=j09.
- Gate 8: pack-overlay passes for mail recovery-notice-delivery and stores evidence with journey_id=j09.
- Gate 9: operator-review passes for mail recovery-notice-delivery and stores evidence with journey_id=j09.
- Gate 10: docs-link-resolves passes for mail recovery-notice-delivery and stores evidence with journey_id=j09.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/mail/IP-journey-j09-recovery-notice-delivery.md` matched `openapi, asyncapi`; contract files `microservices/mail/contracts/openapi/mail.yaml, microservices/mail/contracts/asyncapi/mail-events.yaml, microservices/mail/contracts/proto/mail.proto`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/mail/IP-journey-j09-recovery-notice-delivery.md` matched `emission`; anchors `microservices/mail/manifest.json, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.
