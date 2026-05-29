---
doc_class: Implementation-Plan
ip_id: IP-journey-j05-anonymous-chain-of-custody
journey_id: j05-whistleblower-anonymous-ethics-report
microservice: audit-chain
role: anonymous-chain-of-custody
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
  - docs/user-journeys/j05-whistleblower-anonymous-ethics-report/README.md
  - docs/user-journeys/j05-whistleblower-anonymous-ethics-report/handshake.md
  - docs/user-journeys/j05-whistleblower-anonymous-ethics-report/integration-test-plan.md
---

# IP - j05 - audit-chain - anonymous-chain-of-custody

Goal: implement the audit-chain portion of Anonymous ethics report from SNU Hospital employee so An employee submits an anonymous ethics report through Community while identity proves eligibility but must not bind to submitter.
Binding ADR: ADR-0300. This IP is one independently reviewable slice and must not weaken the common critical-path ADR pack.

## Scope

In scope: anonymous-chain-of-custody, JSON Schema validation, Cedar authorization, audit-chain emission, observability, failure branches, and integration tests for j05.
Out of scope: changing ADRs, changing standards, editing existing PRDs, bypassing Cedar, or adding a new microservice directory.

## Data model

| Object | Storage | Schema | Retention |
|---|---|---|---|
| anonymous-ethics-report | audit-chain.anonymous-chain-of-custody table or event stream | docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json | pack-controlled, minimum audit retention |
| nonbinding-eligibility-proof | audit-chain.anonymous-chain-of-custody table or event stream | docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json | pack-controlled, minimum audit retention |
| report-evidence-envelope | audit-chain.anonymous-chain-of-custody table or event stream | docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json | pack-controlled, minimum audit retention |

## API and event contract

```yaml
openapi: 3.2.0
info:
  title: audit-chain j05 anonymous-chain-of-custody
  version: 1.0.0
paths:
  /journeys/j05/audit-chain/anonymous-chain-of-custody:
    post:
      operationId: j05AuditChainAnonymousChainOfCustody
      x-binding-adr: ADR-0300
      x-audit-required: true
      responses:
        "202":
          description: Accepted and audit-sealed
```

```yaml
asyncapi: 3.1.0
info:
  title: audit-chain j05 events
  version: 1.0.0
channels:
  j05.audit-chain.anonymous-chain-of-custody.accepted:
    address: j05.audit-chain.anonymous-chain-of-custody.accepted
```

## Cedar and policy grammar

The caller-side policy library evaluates before network dispatch where possible. Service-side policy re-evaluates on mutation.

```cedar
permit(principal, action == Action::"j05.execute", resource)
when {
  principal.tenant_id == resource.tenant_id &&
  resource.binding_adr == "ADR-0300" &&
  context.cell_tier >= resource.minimum_cell_tier &&
  context.audit_chain_required == true
};

forbid(principal, action == Action::"j05.execute", resource)
when {
  principal.tenant_id != resource.tenant_id &&
  context.cross_tenant_grant_absent == true
};
```

## Implementation steps

### Step 01 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.1.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 02 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.2.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 03 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.3.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 04 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.4.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 05 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.5.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 06 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.6.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 07 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.7.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 08 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.8.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 09 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.9.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 10 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.10.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 11 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.11.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 12 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.12.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 13 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.13.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 14 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.14.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 15 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.15.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 16 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.16.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 17 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.17.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 18 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.18.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 19 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.19.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 20 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.20.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 21 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.21.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 22 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.22.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 23 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.23.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 24 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.24.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 25 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.25.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 26 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.26.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 27 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.27.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 28 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.28.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 29 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.29.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 30 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.30.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 31 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.31.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 32 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.32.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 33 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.33.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 34 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.34.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 35 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.35.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 36 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.36.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 37 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.37.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 38 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.38.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 39 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.39.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 40 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.40.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 41 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.41.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 42 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.42.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 43 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.43.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 44 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.44.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 45 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.45.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 46 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.46.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 47 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.47.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 48 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.48.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 49 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.49.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 50 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.50.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 51 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.51.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 52 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.52.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 53 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.53.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 54 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.54.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 55 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.55.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 56 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.56.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 57 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.57.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 58 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.58.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 59 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.59.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 60 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.60.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 61 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.61.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 62 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.62.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 63 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.63.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 64 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.64.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 65 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.65.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 66 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.66.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 67 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.67.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 68 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.68.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 69 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.69.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 70 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for anonymous-ethics-report without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/anonymous-ethics-report.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.70.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 71 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for nonbinding-eligibility-proof without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/nonbinding-eligibility-proof.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.71.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 72 - audit-chain anonymous-chain-of-custody slice detail
- Build: add or wire the anonymous-chain-of-custody handler for report-evidence-envelope without changing unrelated audit-chain surfaces.
- Validate: parse docs/user-journeys/j05-whistleblower-anonymous-ethics-report/schemas/report-evidence-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for audit-chain.
- Emit: publish j05.audit-chain.anonymous-chain-of-custody.accepted and seal audit class j05.audit-chain.72.sealed.
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
For j05, the baseline planning rate is 100 journey starts per minute per active cell unless a stricter emergency or compliance pack overrides it.
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
- j05.journey.accepted: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j05.cedar.decision: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j05.audit.sealed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j05.handoff.completed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j05.exception.reviewed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.

Metrics emitted:
- oyatie_j05_accepted_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j05_refused_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j05_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j05_audit_seal_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j05_manual_review_queue_depth: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j05_notification_delivery_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.

Trace shape:
- span 1: community.whistleblower-intake uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 2: audit-chain.anonymous-chain-of-custody uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 3: observability.privacy-preserving-telemetry uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 4: identity.negative-nonbinding-eligibility uses parent trace from the journey accept span and records Cedar decision plus schema version.

## Acceptance gates

- Gate 1: schema-parse passes for audit-chain anonymous-chain-of-custody and stores evidence with journey_id=j05.
- Gate 2: cedar-permit-deny-forbid passes for audit-chain anonymous-chain-of-custody and stores evidence with journey_id=j05.
- Gate 3: audit-seal passes for audit-chain anonymous-chain-of-custody and stores evidence with journey_id=j05.
- Gate 4: trace-cardinality passes for audit-chain anonymous-chain-of-custody and stores evidence with journey_id=j05.
- Gate 5: 10x-load passes for audit-chain anonymous-chain-of-custody and stores evidence with journey_id=j05.
- Gate 6: replay-idempotency passes for audit-chain anonymous-chain-of-custody and stores evidence with journey_id=j05.
- Gate 7: cross-tenant-negative passes for audit-chain anonymous-chain-of-custody and stores evidence with journey_id=j05.
- Gate 8: pack-overlay passes for audit-chain anonymous-chain-of-custody and stores evidence with journey_id=j05.
- Gate 9: operator-review passes for audit-chain anonymous-chain-of-custody and stores evidence with journey_id=j05.
- Gate 10: docs-link-resolves passes for audit-chain anonymous-chain-of-custody and stores evidence with journey_id=j05.

## Wave 15 counterpart evidence note

This IP is checked against `microservices/audit-chain/competitor-parity-matrix.md` and `microservices/audit-chain/feature-parity-matrix-2026-05-20.md`, not against line count. For the `j05 anonymous chain of custody` slice, the relevant counterpart gap is AWS CloudTrail / Google Cloud Audit Logs / Microsoft Purview Audit parity for searchable immutable audit history, plus Oyatie's additional tenant-verifiable Merkle proof path. The GitHub-pinned root and key manifests from `policy/seal-integrity.md` SI-04 and SI-11 are the evidence channel this implementation must preserve; if the slice cannot publish or verify through that channel, it remains below the Wave 15 substance bar.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/audit-chain/IP-journey-j05-anonymous-chain-of-custody.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/audit-chain/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
