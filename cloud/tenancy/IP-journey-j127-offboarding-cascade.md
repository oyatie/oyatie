---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j127-offboarding-cascade
journey_id: j127-dual-tenant-identity-employee-resigns-and-keeps-personal
microservice: tenancy
role: offboarding-cascade
status: draft
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0276-backup-portability-gdpr-art-20
depends_on:
  - microservices/identity/IP-journey-j127-tenant-membership-revocation.md
  - microservices/workflow-engine/IP-journey-j127-offboarding-orchestrator.md
date: 2026-05-20
owner_team: axis-tenancy + axis-hr
parallel_work_compatibility: Independent of j126 cross-tenant permit grant + j128 personal-tenant workflows
---

# IP-journey-j127-offboarding-cascade — Tenancy µservice: per-tenant offboarding cascade entry point

## Goal

Implement tenancy µservice surfaces that:

1. Validate offboarding-cascade authorization for an HR admin.
2. Hand off to workflow-engine for the multi-µservice cascade.
3. Revoke cross-tenant permits the leaver held.
4. Emit DSAR-portability link if requested per ADR-0276.

## Data model

```sql
-- Migration: 2026-05-20-001-offboarding-records.sql

CREATE TABLE offboarding_records (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id TEXT NOT NULL,
  principal_id TEXT NOT NULL,
  triggered_by_principal_id TEXT NOT NULL,
  workflow_id TEXT NOT NULL,
  triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  effective_at TIMESTAMPTZ NOT NULL,
  reason TEXT NOT NULL,
  drive_transfer_target TEXT NOT NULL,
  dsar_email TEXT,
  cascade_completed_at TIMESTAMPTZ,
  status TEXT NOT NULL DEFAULT 'PENDING'
    CHECK (status IN ('PENDING','IN_PROGRESS','COMPLETED','FAILED','PARTIAL'))
);

CREATE INDEX idx_offboarding_status ON offboarding_records (status, effective_at);
```

## API surface

```protobuf
service TenancyOffboarding {
  rpc TriggerOffboardingCascade (TriggerOffboardingCascadeRequest)
      returns (TriggerOffboardingCascadeResponse);
  rpc GetOffboardingStatus (GetOffboardingStatusRequest)
      returns (GetOffboardingStatusResponse);
}

message TriggerOffboardingCascadeRequest {
  string tenant_id = 1;
  string principal_id = 2;
  google.protobuf.Timestamp effective_at = 3;
  string drive_transfer_target = 4;
  string reason = 5;
  string dsar_email = 6;
}
```

## Files to author

| File | Purpose | Lines |
|---|---|---:|
| `microservices/tenancy/src/offboarding/trigger.rs` | gRPC entry point | ~240 |
| `microservices/tenancy/src/offboarding/cross_tenant_permit_cleanup.rs` | Revoke leaver's cross-tenant permits | ~200 |
| `microservices/tenancy/src/offboarding/dsar_link_issuer.rs` | DSAR-portability link generation per ADR-0276 | ~180 |
| `microservices/tenancy/policy/offboarding-trigger.cedar` | Cedar permit (HR admin only) | ~30 |
| `microservices/tenancy/contracts/proto/offboarding.proto` | gRPC defs | ~100 |
| `microservices/tenancy/db/migrations/2026-05-20-001-offboarding-records.sql` | DDL | ~40 |
| `microservices/tenancy/runbooks/offboarding-cascade-failure-recovery.md` | Runbook | ~160 |
| `microservices/tenancy/runbooks/dsar-portability-export-mid-offboarding.md` | Runbook | ~140 |
| `microservices/tenancy/tests/integration/offboarding_test.rs` | Integration tests | ~360 |
| `microservices/tenancy/dashboards/offboarding-cascade-health.json` | Grafana | ~100 |
| `microservices/tenancy/slos/offboarding-cascade-completion.openslo.yaml` | SLO ≤30s p99 | ~40 |

Total approximate: ~1,590 lines.

## Cedar fragments

```cedar
// offboarding-trigger.cedar
permit (
  principal is User,
  action == Action::"tenancy.trigger_offboarding_cascade",
  resource is TenantMembership
) when {
  principal.audience_type == "B2B_HR_ADMIN" &&
  principal.tenant == resource.tenant_id &&
  // CRITICAL: can only offboard from the same tenant the HR admin scopes to
  context.effective_at >= context.now
};
```

## Integration contracts

| Contract | Direction | Notes |
|---|---|---|
| `workflow-engine.StartWorkflow` | tenancy → workflow-engine | Cascade orchestration |
| `tenancy.RevokeCrossTenantPermit` (self) | internal | Revoke leaver's outbound permits |
| `audit-chain.EmitSealed` | tenancy → audit-chain | Workflow-class events |
| `comms-email.SendDsarPortabilityLink` | tenancy → comms-email | DSAR link delivery |

## Latency budget

- `TriggerOffboardingCascade`: ≤300ms p99 (synchronous portion)
- Full cascade (via workflow-engine): ≤30s p99

## Test plan

- Test A.1, A.2 — cascade triggers correctly
- Test C.1 — cross-tenant permit revoked
- Test E.2 — drive transfer failure escalates

## Observability emissions

- `oya_tenancy_offboarding_triggered_total{tenant_id,reason}`
- `oya_tenancy_offboarding_completed_total{tenant_id,status}`
- `oya_tenancy_offboarding_cross_tenant_permits_revoked_total`
- `oya_tenancy_dsar_link_issued_total`

## Acceptance criteria

- All migrations apply forward + rollback.
- Cedar permits parse + validate.
- Integration tests pass.

## Cross-references

- `docs/user-journeys/j127-*/handshake.md` §3 + §5
- ADR-0311 + ADR-0276

## Completion expansion — j127 tenancy IP rigor pass

Journey context: employee resignation where work access is revoked and personal tenant survives.
Service role: tenant membership, sub-scope, residency, and cross-tenant grant boundary.
Mapped services in this journey: identity, tenancy, messenger, mail, drive, workflow-engine.
ADR anchors: ADR-0244, ADR-0299, ADR-0311, ADR-0313, ADR-0317, ADR-0320.
This IP is sized as a single reviewable implementation slice and remains compatible with the 56-µservice flat layout.

Implementation task 001: in tenancy, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 001: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 001: add property coverage proving tenancy and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 001: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 002: in tenancy, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 002: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 002: add contract coverage proving tenancy and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 002: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 003: in tenancy, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 003: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 003: add integration coverage proving tenancy and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 003: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 004: in tenancy, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 004: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 004: add replay coverage proving tenancy and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 004: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 005: in tenancy, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 005: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 005: add load coverage proving tenancy and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 005: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 006: in tenancy, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 006: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 006: add chaos coverage proving tenancy and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 006: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 007: in tenancy, define the dashboard projection change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 007: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 007: add negative authorization coverage proving tenancy and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 007: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 008: in tenancy, define the runbook hook change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 008: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 008: add multi-region coverage proving tenancy and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 008: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 009: in tenancy, define the integration fixture change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 009: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 009: add pack-overlay coverage proving tenancy and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 009: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 010: in tenancy, define the domain model change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 010: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 010: add unit coverage proving tenancy and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 010: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 01: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 011: in tenancy, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 011: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 011: add property coverage proving tenancy and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 011: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 012: in tenancy, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 012: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 012: add contract coverage proving tenancy and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 012: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 013: in tenancy, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 013: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 013: add integration coverage proving tenancy and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 013: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 014: in tenancy, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 014: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 014: add replay coverage proving tenancy and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 014: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 015: in tenancy, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 015: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 015: add load coverage proving tenancy and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 015: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 016: in tenancy, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 016: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 016: add chaos coverage proving tenancy and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 016: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 017: in tenancy, define the dashboard projection change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 017: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 017: add negative authorization coverage proving tenancy and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 017: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 018: in tenancy, define the runbook hook change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 018: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 018: add multi-region coverage proving tenancy and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 018: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 019: in tenancy, define the integration fixture change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 019: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 019: add pack-overlay coverage proving tenancy and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 019: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 020: in tenancy, define the domain model change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 020: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 020: add unit coverage proving tenancy and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 020: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 02: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 021: in tenancy, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 021: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 021: add property coverage proving tenancy and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 021: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 022: in tenancy, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 022: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 022: add contract coverage proving tenancy and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 022: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 023: in tenancy, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 023: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 023: add integration coverage proving tenancy and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 023: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 024: in tenancy, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 024: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 024: add replay coverage proving tenancy and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 024: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 025: in tenancy, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 025: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 025: add load coverage proving tenancy and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 025: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 026: in tenancy, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 026: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 026: add chaos coverage proving tenancy and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 026: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 027: in tenancy, define the dashboard projection change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 027: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 027: add negative authorization coverage proving tenancy and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 027: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 028: in tenancy, define the runbook hook change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 028: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 028: add multi-region coverage proving tenancy and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 028: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 029: in tenancy, define the integration fixture change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 029: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 029: add pack-overlay coverage proving tenancy and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 029: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 030: in tenancy, define the domain model change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 030: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 030: add unit coverage proving tenancy and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 030: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 03: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 031: in tenancy, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 031: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 031: add property coverage proving tenancy and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 031: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 032: in tenancy, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 032: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 032: add contract coverage proving tenancy and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 032: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 033: in tenancy, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 033: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 033: add integration coverage proving tenancy and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 033: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 034: in tenancy, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 034: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 034: add replay coverage proving tenancy and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 034: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 035: in tenancy, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 035: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 035: add load coverage proving tenancy and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 035: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 036: in tenancy, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 036: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 036: add chaos coverage proving tenancy and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 036: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 037: in tenancy, define the dashboard projection change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 037: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 037: add negative authorization coverage proving tenancy and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 037: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 038: in tenancy, define the runbook hook change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 038: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 038: add multi-region coverage proving tenancy and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 038: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 039: in tenancy, define the integration fixture change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 039: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 039: add pack-overlay coverage proving tenancy and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 039: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 040: in tenancy, define the domain model change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 040: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 040: add unit coverage proving tenancy and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 040: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 04: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 041: in tenancy, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 041: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 041: add property coverage proving tenancy and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 041: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 042: in tenancy, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 042: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 042: add contract coverage proving tenancy and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 042: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 043: in tenancy, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 043: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 043: add integration coverage proving tenancy and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 043: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 044: in tenancy, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 044: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 044: add replay coverage proving tenancy and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 044: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 045: in tenancy, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 045: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 045: add load coverage proving tenancy and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 045: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 046: in tenancy, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 046: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 046: add chaos coverage proving tenancy and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 046: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 047: in tenancy, define the dashboard projection change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 047: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 047: add negative authorization coverage proving tenancy and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 047: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 048: in tenancy, define the runbook hook change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 048: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 048: add multi-region coverage proving tenancy and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 048: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 049: in tenancy, define the integration fixture change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 049: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 049: add pack-overlay coverage proving tenancy and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 049: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 050: in tenancy, define the domain model change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 050: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 050: add unit coverage proving tenancy and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 050: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 05: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 051: in tenancy, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 051: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 051: add property coverage proving tenancy and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 051: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 052: in tenancy, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 052: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 052: add contract coverage proving tenancy and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 052: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 053: in tenancy, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 053: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 053: add integration coverage proving tenancy and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 053: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 054: in tenancy, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 054: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 054: add replay coverage proving tenancy and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 054: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 055: in tenancy, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 055: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 055: add load coverage proving tenancy and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 055: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 056: in tenancy, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 056: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 056: add chaos coverage proving tenancy and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 056: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 057: in tenancy, define the dashboard projection change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 057: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 057: add negative authorization coverage proving tenancy and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 057: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 058: in tenancy, define the runbook hook change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 058: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 058: add multi-region coverage proving tenancy and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 058: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 059: in tenancy, define the integration fixture change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 059: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 059: add pack-overlay coverage proving tenancy and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 059: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 060: in tenancy, define the domain model change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 060: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 060: add unit coverage proving tenancy and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 060: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 06: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 061: in tenancy, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 061: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 061: add property coverage proving tenancy and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 061: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 062: in tenancy, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 062: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 062: add contract coverage proving tenancy and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 062: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 063: in tenancy, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 063: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 063: add integration coverage proving tenancy and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 063: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 064: in tenancy, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 064: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 064: add replay coverage proving tenancy and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 064: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 065: in tenancy, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 065: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 065: add load coverage proving tenancy and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 065: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 066: in tenancy, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 066: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 066: add chaos coverage proving tenancy and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 066: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/tenancy/IP-journey-j127-offboarding-cascade.md` matched `.proto`; contract files `microservices/tenancy/contracts/openapi/tenancy.yaml, microservices/tenancy/contracts/asyncapi/tenant-events.yaml, microservices/tenancy/contracts/proto/tenancy.proto`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/tenancy/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/tenancy/IP-journey-j127-offboarding-cascade.md` matched `p99, SLO, multi-region`; anchors `microservices/tenancy/runbooks/dr-pair-promotion-drill.md, crates/oya-tenancy-api/src/lib.rs`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/tenancy/IP-journey-j127-offboarding-cascade.md` matched `emission`; anchors `microservices/tenancy/manifest.json, crates/oya-tenancy-api/src/lib.rs`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.
