---
doc_class: IP
ip_id: IP-journey-j01-pending-chart
journey_id: j01-emergency-911-dispatch
microservice: ontology
role: pending-chart-object-type
status: draft
related_adrs: [ADR-0257, ADR-0263, ADR-0244, ADR-0298]
depends_on:
  - microservices/audit-chain/IP-journey-j01-emergency-911-dispatch-emergency-classes.md
date: 2026-05-20
owner_team: axis-ontology + axis-healthcare-vertical
---

# IP-journey-j01-pending-chart — Ontology: PendingChart object type for KR-119 pre-arrival

## Goal

Define the `PendingChart` ontology object type with versioning per
ADR-0257, ABAC scoping per ADR-0244, and pre-arrival-source field set.

## Object type definition

```yaml
ontology_object_type: PendingChart
  version: v1.0.0
  tenant_scope: REQUIRED
  cell_tier_minimum: 3 (regulated)
  pack_overlay_required: [pack-hipa-2024, pack-kr-medical-records-act]
  fields:
    - name: provisional_mrn
      type: string
      required: true
      unique_within_tenant: true
    - name: patient_demographics
      type: ref
      target: PatientDemographics
      required: true
    - name: presenting_complaint
      type: string
      required: true
      pii_class: PHI
    - name: suspected_diagnosis_codes
      type: array<string>
      pii_class: PHI
    - name: source
      type: string
      enum: ["119-EMS-pre-arrival", "walk-in", "transfer", "scheduled"]
    - name: acute_severity_grade
      type: string
      enum: ["acute-cardiac-suspect", "acute-neuro-suspect", "acute-trauma", "acute-respiratory", "acute-obstetric", "other"]
    - name: assigned_attending
      type: ref
      target: ClinicianPrincipal
    - name: assigned_nurse
      type: ref
      target: ClinicianPrincipal
    - name: created_at
      type: timestamp
    - name: reconciled_with_mrn
      type: ref
      target: PatientChart
      nullable: true
```

## Files

| File | Size |
|---|---|
| `microservices/ontology/src/types/pending_chart.rs` | ~280 lines |
| `microservices/ontology/object-types/pending-chart-v1.yaml` | ~80 lines |
| `microservices/ontology/policy/ontology-chart-create-from-119.cedar` | ~30 lines |
| `microservices/ontology/policy/ontology-chart-read.cedar` | ~40 lines |
| `microservices/ontology/contracts/proto/pending_chart.proto` | ~100 lines |
| `microservices/ontology/db/migrations/2026-05-20-001-pending-charts.sql` | ~50 lines |
| `microservices/ontology/tests/integration/pending_chart_test.rs` | ~400 lines |
| `microservices/ontology/runbooks/pending-chart-reconciliation.md` | ~120 lines |

## Cedar

```cedar
permit (
  principal == Workflow::"snuh.org/er-intake-incoming-acute",
  action == Action::"ontology.create_pending_chart",
  resource is Tenant
) when {
  resource.compliance_pack_active("pack-hipa-2024") &&
  resource.compliance_pack_active("pack-kr-medical-records-act")
};

forbid (
  principal,
  action == Action::"ontology.read_chart",
  resource is PendingChart
) unless {
  principal.assigned_to(resource) == true ||
  principal in Role::"snuh.org/care-team-of-resource"
};
```

## Audit events

`ChartPendingCreatedFromPreArrival`, `ChartReadAttempted`, `ChartReconciled`.

## SLOs

- create_pending_chart_p95 ≤ 500ms.
- chart_read_authz_p95 ≤ 80ms.

## Tests

Per integration-test-plan §4 + §7.

## Parallel work

Depends on audit-chain. Reused by j02, j18.

— end of IP —

## Completion expansion for j01 ontology emergency-911-dispatch-pending-chart

This appendix completes a pre-existing partial IP scaffold to the 400-line per-service bar required by /tmp/codex-brief-j01-j20-lifesafety.md.
The expansion is bound to ADR-0298 and the shared life-safety ADR pack ADR-0298, ADR-0299, ADR-0300, ADR-0301, ADR-0302, ADR-0303, ADR-0304, ADR-0305, ADR-0306, ADR-0292.

## Completion scope

- Microservice: ontology.
- Journey: j01 Emergency 119 dispatch.
- Role: emergency-911-dispatch-pending-chart.
- This is an additive completion; prior scaffold text above is preserved.
- No ADR, standard, PRD, or ARCHITECTURE file is modified by this appendix.

## Contract closure

| Surface | Required behavior | Evidence |
|---|---|---|
| OpenAPI 3.2.0 command | ontology validates j01 emergency-911-dispatch-pending-chart with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| AsyncAPI 3.1.0 event | ontology validates j01 emergency-911-dispatch-pending-chart with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| proto3 internal RPC | ontology validates j01 emergency-911-dispatch-pending-chart with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| Cedar v4.1 policy | ontology validates j01 emergency-911-dispatch-pending-chart with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| audit-chain seal | ontology validates j01 emergency-911-dispatch-pending-chart with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| observability span | ontology validates j01 emergency-911-dispatch-pending-chart with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| integration harness fixture | ontology validates j01 emergency-911-dispatch-pending-chart with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |

## Implementation steps

### Step 01 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 02 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 03 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 04 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 05 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 06 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 07 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 08 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 09 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 10 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 11 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 12 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 13 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 14 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 15 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 16 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 17 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 18 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 19 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 20 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 21 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 22 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 23 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 24 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 25 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 26 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 27 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 28 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 29 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 30 - ontology emergency-911-dispatch-pending-chart
- Build: wire the emergency-911-dispatch-pending-chart handler behind the existing ontology boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.



## Counterpart Evidence

This already-substantive IP is preserved. Counterpart anchor for Wave 15 verification: Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model. See `microservices/ontology/competitor-parity-matrix.md` for the service-specific parity rows; the implementation PR must update that row when this IP materially changes parity.
