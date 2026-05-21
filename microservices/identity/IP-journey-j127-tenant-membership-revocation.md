---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j127-tenant-membership-revocation
journey_id: j127-dual-tenant-identity-employee-resigns-and-keeps-personal
microservice: identity
role: tenant-membership-revocation
status: draft
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0188-passkey-webauthn-as-canonical-auth
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0263-observability-emission-contract
depends_on:
  - microservices/identity/IP-017-multi-context-principal-resolver.md
  - microservices/identity/IP-journey-j126-fedramp-3pao-cross-tenant-resolver.md
date: 2026-05-20
owner_team: axis-identity + axis-hr-offboarding
parallel_work_compatibility: |
  Independent of j126 cross-tenant resolver (j126 reads memberships;
  j127 transitions them). Shares the credential-handle-roster
  primitive with j126. Can be authored in parallel with j128, j130
  multi-tenant operations.
---

# IP-journey-j127-tenant-membership-revocation — Identity µservice: per-tenant credential handle revocation grammar

## Goal

Implement identity µservice surfaces that revoke a tenant membership
without affecting the principal's OTHER tenant memberships. Three
RPCs:

1. **`RevokeTenantMembership`** — sets `tenant_memberships.status =
   'REVOKED'` for ONE row, scoped to one tenant; idempotent.
2. **`RevokeCredentialHandle`** — sets `webauthn_credentials.active =
   FALSE` for ONE credential handle, leaving other handles on the
   same hardware-key active.
3. **`ListActiveTenantMemberships`** — returns only ACTIVE memberships;
   drives the context-picker UX.

These surfaces are the **identity-µservice contribution** to ADR-0311
§B-3 per-tenant-revocation invariant.

## Data model

```sql
-- Migration: 2026-05-20-001-tenant-membership-status.sql

ALTER TABLE tenant_memberships
  ADD COLUMN status TEXT NOT NULL DEFAULT 'ACTIVE'
    CHECK (status IN ('ACTIVE','SUSPENDED','REVOKED','SOFT_DELETE')),
  ADD COLUMN revoked_at TIMESTAMPTZ,
  ADD COLUMN revoked_by_principal_id TEXT,
  ADD COLUMN revocation_reason TEXT,
  ADD COLUMN revocation_workflow_id TEXT;

CREATE INDEX idx_tenant_memberships_status_principal
  ON tenant_memberships (principal_id, status);

-- Refresh the materialized view used by context-picker so it filters
-- on status = 'ACTIVE'.

DROP MATERIALIZED VIEW IF EXISTS multi_tenant_membership_index;
CREATE MATERIALIZED VIEW multi_tenant_membership_index AS
  SELECT
    wc.credential_id,
    array_agg(DISTINCT tm.tenant_id) AS tenant_ids,
    array_agg(DISTINCT tm.tenant_id || '|' || COALESCE(ata.audience_type,'unspecified')) AS tenant_audience_pairs
  FROM webauthn_credentials wc
  JOIN tenant_memberships tm ON tm.principal_id = wc.principal_id
  LEFT JOIN audience_type_assignments ata
    ON ata.principal_id = tm.principal_id AND ata.tenant_id = tm.tenant_id
    AND ata.active = TRUE
  WHERE wc.active = TRUE
    AND tm.status = 'ACTIVE'   -- KEY: revoked memberships excluded
  GROUP BY wc.credential_id;

-- Migration: 2026-05-20-002-credential-handle-per-tenant.sql

ALTER TABLE webauthn_credentials
  ADD COLUMN tenant_id TEXT NOT NULL DEFAULT 'platform-bootstrap';

-- Each credential row is bound to ONE tenant.
CREATE INDEX idx_webauthn_credentials_active_tenant
  ON webauthn_credentials (active, tenant_id);

CREATE INDEX idx_webauthn_credentials_principal_tenant
  ON webauthn_credentials (principal_id, tenant_id);
```

## API surface

```protobuf
service IdentityMembershipRevocation {
  rpc RevokeTenantMembership (RevokeTenantMembershipRequest)
      returns (RevokeTenantMembershipResponse);
  rpc RevokeCredentialHandle (RevokeCredentialHandleRequest)
      returns (RevokeCredentialHandleResponse);
  rpc ListActiveTenantMemberships (ListActiveTenantMembershipsRequest)
      returns (ListActiveTenantMembershipsResponse);
  rpc EnrollNewCredentialHandle (EnrollNewCredentialHandleRequest)
      returns (EnrollNewCredentialHandleResponse);
}

message RevokeTenantMembershipRequest {
  string tenant_id = 1;       // SCOPE: only this tenant
  string principal_id = 2;
  string reason = 3;
  string workflow_id = 4;
}

message RevokeTenantMembershipResponse {
  google.protobuf.Timestamp revoked_at = 1;
  // Other tenant memberships for the same principal are NOT affected.
  // The response includes a count for confirmation.
  int32 other_active_memberships_unchanged = 2;
}

message RevokeCredentialHandleRequest {
  string credential_handle_id = 1;
  string reason = 2;
}

message ListActiveTenantMembershipsRequest {
  string principal_id = 1;
}

message ListActiveTenantMembershipsResponse {
  repeated TenantMembership memberships = 1;
}
```

## Files to author

| File | Purpose | Approx. lines |
|---|---|---:|
| `microservices/identity/src/revocation/membership_revoke.rs` | RevokeTenantMembership impl | ~240 |
| `microservices/identity/src/revocation/credential_revoke.rs` | RevokeCredentialHandle impl | ~180 |
| `microservices/identity/src/enrollment/new_handle.rs` | EnrollNewCredentialHandle impl | ~220 |
| `microservices/identity/policy/membership-revoke.cedar` | Cedar permit | ~30 |
| `microservices/identity/policy/credential-revoke.cedar` | Cedar permit | ~30 |
| `microservices/identity/policy/credential-enroll.cedar` | Cedar permit | ~30 |
| `microservices/identity/contracts/proto/membership_revocation.proto` | gRPC defs | ~140 |
| `microservices/identity/db/migrations/2026-05-20-001-tenant-membership-status.sql` | DDL | ~60 |
| `microservices/identity/db/migrations/2026-05-20-002-credential-handle-per-tenant.sql` | DDL | ~40 |
| `microservices/identity/runbooks/tenant-membership-revoke-emergency.md` | Runbook | ~150 |
| `microservices/identity/runbooks/credential-handle-roster-audit.md` | Runbook | ~130 |
| `microservices/identity/tests/integration/membership_revocation_test.rs` | Integration tests | ~420 |
| `microservices/identity/dashboards/tenant-membership-lifecycle.json` | Grafana | ~100 |
| `microservices/identity/slos/membership-revoke-latency.openslo.yaml` | SLO ≤500ms p99 | ~40 |

Total approximate new code + content: ~1,810 lines.

## Cedar fragments

```cedar
// membership-revoke.cedar
permit (
  principal == Service::"workflow-engine",
  action == Action::"identity.revoke_tenant_membership",
  resource is TenantMembership
) when {
  context.workflow_class == "offboarding" &&
  resource.tenant_id == context.target_tenant_id &&
  // CRITICAL: cannot affect OTHER tenant memberships of the same principal
  resource.principal_id == context.target_principal_id
};

// credential-revoke.cedar
permit (
  principal == Service::"workflow-engine",
  action == Action::"identity.revoke_credential_handle",
  resource is WebAuthnCredential
) when {
  resource.tenant_id == context.target_tenant_id
  // CRITICAL: scope is per-handle, not per-hardware-key
};

// credential-enroll.cedar
permit (
  principal is User,
  action == Action::"identity.enroll_new_credential_handle",
  resource is User
) when {
  principal.id == resource.id &&
  // New handle is bound to ONE new tenant
  context.target_tenant_id != null &&
  // Per-tenant onboarding workflow has authorized the enrollment
  context.onboarding_workflow_active == true
};
```

## Cross-µservice handshake

### Phase 3.1 (handshake.md) — Revocation

When `RevokeTenantMembership` is called:

1. Cedar permit verifies caller is workflow-engine and target row is
   the correct one.
2. UPDATE `tenant_memberships SET status='REVOKED', revoked_at=NOW(), ...`.
3. UPDATE `webauthn_credentials SET active=FALSE` for handles bound to
   this tenant (ONE handle for ONE tenant).
4. Emit `TenantMembershipRevoked` to the revoked tenant's audit-chain.
5. Refresh `multi_tenant_membership_index` materialized view.
6. Return success + count of OTHER active memberships unchanged.

### Phase 6.2 (handshake.md) — New credential enrollment

When `EnrollNewCredentialHandle` is called:

1. Cedar permit verifies caller is the user enrolling.
2. WebAuthn create-credential ceremony per ADR-0188 §C-create.
3. INSERT new row in `webauthn_credentials` with `tenant_id` set.
4. INSERT row in `tenant_memberships` with `status='ACTIVE'`.
5. Refresh materialized view.
6. Emit `TenantMembershipCreated` to the new tenant's audit-chain.

## Integration contracts

| Contract | Direction | Notes |
|---|---|---|
| `audit-chain.EmitSealed` | identity → audit-chain (single tenant) | Per revocation/creation event |
| `observability.PushOTLP` | identity → observability | Lifecycle counters |
| `tenancy.OnTenantMembershipChange` | identity → tenancy | Tenant admin notification |

## Latency budget

| RPC | p50 | p95 | p99 | Hard cap |
|---|---:|---:|---:|---:|
| `RevokeTenantMembership` | 80ms | 180ms | 320ms | 500ms |
| `RevokeCredentialHandle` | 40ms | 90ms | 150ms | 250ms |
| `ListActiveTenantMemberships` | 30ms | 60ms | 100ms | 180ms |
| `EnrollNewCredentialHandle` | 280ms | 540ms | 820ms | 1.5s |

## Parallel work compatibility

- j126 uses `ListActiveTenantMemberships` (read-only) — independent.
- j128 uses `EnrollNewCredentialHandle` for cross-context bridging.
- j130 uses `ListActiveTenantMemberships` for whistleblower verification.

## Test plan summary

- Test A.1 — Revocation transitions to REVOKED
- Test A.2 — Personal-tenant membership unchanged
- Test B.4 — Personal credential handle remains ACTIVE
- Test B.5 — Work credential handle REVOKED
- Test B.6 — Context-picker hides revoked tenant
- Test D.1 — New credential handle on Bristlecone
- Test D.2 — Monday picker shows two

## Observability emissions

- `oya_identity_tenant_membership_revoked_total{tenant_id,reason}`
- `oya_identity_credential_handle_revoked_total{tenant_id,reason}`
- `oya_identity_tenant_membership_active_gauge{tenant_id}`
- `oya_identity_multi_tenant_membership_index_refresh_latency_ms`

## Acceptance criteria

- All migrations apply forward + rollback.
- All Cedar fragments parse + validate.
- Integration tests pass.
- Latency budget held under 1k QPS.

## Cross-references

- `docs/user-journeys/j127-*/handshake.md`
- ADR-0311 §B-3
- ADR-0188 §D credential handle roster

## Completion expansion — j127 identity IP rigor pass

Journey context: employee resignation where work access is revoked and personal tenant survives.
Service role: principal resolver, audience-type classifier, passkey continuity, and tenant-membership boundary.
Mapped services in this journey: identity, tenancy, messenger, mail, drive, workflow-engine.
ADR anchors: ADR-0244, ADR-0299, ADR-0311, ADR-0313, ADR-0317, ADR-0320.
This IP is sized as a single reviewable implementation slice and remains compatible with the 56-µservice flat layout.

Implementation task 001: in identity, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 001: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 001: add property coverage proving identity and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 001: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 002: in identity, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 002: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 002: add contract coverage proving identity and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 002: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 003: in identity, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 003: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 003: add integration coverage proving identity and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 003: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 004: in identity, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 004: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 004: add replay coverage proving identity and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 004: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 005: in identity, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 005: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 005: add load coverage proving identity and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 005: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 006: in identity, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 006: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 006: add chaos coverage proving identity and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 006: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 007: in identity, define the dashboard projection change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 007: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 007: add negative authorization coverage proving identity and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 007: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 008: in identity, define the runbook hook change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 008: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 008: add multi-region coverage proving identity and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 008: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 009: in identity, define the integration fixture change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 009: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 009: add pack-overlay coverage proving identity and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 009: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 010: in identity, define the domain model change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 010: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 010: add unit coverage proving identity and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 010: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 01: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 011: in identity, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 011: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 011: add property coverage proving identity and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 011: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 012: in identity, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 012: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 012: add contract coverage proving identity and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 012: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 013: in identity, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 013: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 013: add integration coverage proving identity and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 013: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 014: in identity, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 014: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 014: add replay coverage proving identity and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 014: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 015: in identity, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 015: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 015: add load coverage proving identity and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 015: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 016: in identity, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 016: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 016: add chaos coverage proving identity and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 016: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 017: in identity, define the dashboard projection change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 017: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 017: add negative authorization coverage proving identity and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 017: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 018: in identity, define the runbook hook change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 018: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 018: add multi-region coverage proving identity and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 018: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 019: in identity, define the integration fixture change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 019: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 019: add pack-overlay coverage proving identity and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 019: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 020: in identity, define the domain model change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 020: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 020: add unit coverage proving identity and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 020: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 02: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 021: in identity, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 021: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 021: add property coverage proving identity and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 021: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 022: in identity, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 022: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 022: add contract coverage proving identity and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 022: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 023: in identity, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 023: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 023: add integration coverage proving identity and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 023: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 024: in identity, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 024: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 024: add replay coverage proving identity and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 024: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 025: in identity, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 025: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 025: add load coverage proving identity and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 025: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 026: in identity, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 026: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 026: add chaos coverage proving identity and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 026: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 027: in identity, define the dashboard projection change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 027: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 027: add negative authorization coverage proving identity and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 027: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 028: in identity, define the runbook hook change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 028: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 028: add multi-region coverage proving identity and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 028: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 029: in identity, define the integration fixture change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 029: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 029: add pack-overlay coverage proving identity and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 029: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 030: in identity, define the domain model change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 030: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 030: add unit coverage proving identity and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 030: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 03: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 031: in identity, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 031: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 031: add property coverage proving identity and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 031: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 032: in identity, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 032: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 032: add contract coverage proving identity and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 032: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 033: in identity, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 033: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 033: add integration coverage proving identity and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 033: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 034: in identity, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 034: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 034: add replay coverage proving identity and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 034: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 035: in identity, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 035: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 035: add load coverage proving identity and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 035: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 036: in identity, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 036: identity MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.

## Counterpart references - journey-j127-tenant-membership-revocation

- Counterpart class: principal / context resolution.
- Palantir Foundry is the closest counterpart for explicit organization-context access control; this IP adapts that property to identity by requiring an explicit principal/context envelope before downstream services can read, mutate, or disclose tenant data.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `microservices/identity/PRD.md`, `microservices/identity/manifest.json`, and the contract/policy files cited above.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/identity/IP-journey-j127-tenant-membership-revocation.md` matched `SLO, multi-region, p99`.
- Numeric target: `rto_p99_seconds=30`, `rpo_p99_seconds=0` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), PCI-DSS-L1-v4(86400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/identity/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/webauthn-authenticate-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`, `microservices/identity/policy/cedar-acr-predicates.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/identity/IP-journey-j127-tenant-membership-revocation.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/identity/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
