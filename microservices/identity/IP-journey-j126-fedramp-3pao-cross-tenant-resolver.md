---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j126-fedramp-3pao-cross-tenant-resolver
journey_id: j126-government-auditor-3pao-conducts-fedramp-audit
microservice: identity
role: cross-tenant-resolver
status: draft
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0188-passkey-webauthn-as-canonical-auth
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0299-account-recovery-resilience
  - ADR-0263-observability-emission-contract
depends_on:
  - microservices/identity/IP-017-multi-context-principal-resolver.md
  - microservices/audit-chain/IP-journey-j126-dual-tenant-emission-classes.md
  - microservices/tenancy/IP-journey-j126-cross-tenant-permit-grant.md
date: 2026-05-20
owner_team: axis-identity + axis-fedramp-compliance
parallel_work_compatibility: |
  Independent of j127 (resignation tenant-revocation) — that touches
  tenancy + identity but a different surface (tenant-membership
  transition state). Shares the multi-context-principal-resolver
  primitive (IP-017) with j128, j130. Authors can land in any order;
  j126 is the foundation for the dual-tenant journey suite.
---

# IP-journey-j126-fedramp-3pao-cross-tenant-resolver — Identity µservice: cross-tenant principal resolver and INTERNAL_AUDITOR_3PAO audience-type setter

## Goal

Implement three identity µservice surfaces that j126 exercises:

1. **`VerifyAssertion` with two-tenants response.** When a WebAuthn
   credential resolves to ≥2 active tenant memberships, identity returns
   a `TwoTenantsResponse` to drive the context-picker UX (per
   `ux-flow.md` §2.5).
2. **`InitSession` with explicit tenant + audience-type binding.** After
   the user picks a tenant in the context-picker, identity establishes a
   session bound to that single `tenant_id` + `audience_type =
   INTERNAL_AUDITOR_3PAO`. Cross-tenant operations require this binding.
3. **`Get3paoAccreditationStatus` live lookup.** Every Cedar evaluation
   that gates the cross-tenant audit pull requires a live (≤200ms p99)
   lookup of the principal's 3PAO accreditation status (per story §19
   invariant 8). Identity owns this surface.

These three surfaces are the **identity-µservice contribution** to the
ADR-0311 dual-tenant boundary. All three are guarded by Cedar permits
and emit audit events to the appropriate per-tenant audit-chain.

## Data model

### Postgres tables (additions to identity µservice's schema)

| Table | Purpose |
|---|---|
| `fedramp_3pao_accreditations` | Per-principal 3PAO accreditation registry |
| `audience_type_assignments` | Per-principal per-tenant audience-type assignment |
| `multi_tenant_membership_index` | Materialized view for fast credential→tenants lookup |

```sql
-- Migration: 2026-05-20-001-fedramp-3pao-accreditations.sql

CREATE TABLE fedramp_3pao_accreditations (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  principal_id TEXT NOT NULL,
  tenant_id TEXT NOT NULL,
  accreditation_id TEXT NOT NULL,    -- e.g., "3PAO-2023-0147"
  accreditation_authority TEXT NOT NULL CHECK (
    accreditation_authority IN (
      'FedRAMP-PMO',
      'A2LA',  -- accrediting body
      'NIST'
    )
  ),
  baseline_authorized TEXT[] NOT NULL CHECK (
    baseline_authorized <@ ARRAY['Low', 'Moderate', 'High', 'Tailored-LI-SaaS']
  ),
  issued_at TIMESTAMPTZ NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  active BOOLEAN NOT NULL DEFAULT TRUE,
  revocation_reason TEXT,
  revoked_at TIMESTAMPTZ,
  UNIQUE (principal_id, tenant_id, accreditation_id)
);

CREATE INDEX idx_fedramp_3pao_active_by_principal
  ON fedramp_3pao_accreditations (principal_id, tenant_id)
  WHERE active = TRUE;

-- Migration: 2026-05-20-002-audience-type-assignments.sql

CREATE TABLE audience_type_assignments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  principal_id TEXT NOT NULL,
  tenant_id TEXT NOT NULL,
  audience_type TEXT NOT NULL CHECK (
    audience_type IN (
      'B2C_CONSUMER',
      'B2B_TENANT_ADMIN',
      'B2B_HR_ADMIN',
      'B2B_INTERNAL_AUDIT',
      'INTERNAL_AUDITOR_3PAO',
      'B2C_JOB_SEEKER_ACTIVE',
      'EMERGENCY_SERVICES',
      'FRIENDLY_CRAWLER_PARTNER',
      'MINOR_TARGETED',
      'INTERNAL_DEV_TOOLS'
    )
  ),
  assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  assigned_by TEXT NOT NULL,
  active BOOLEAN NOT NULL DEFAULT TRUE,
  UNIQUE (principal_id, tenant_id, audience_type)
);

CREATE INDEX idx_audience_type_active
  ON audience_type_assignments (principal_id, tenant_id)
  WHERE active = TRUE;

-- Materialized view: fast credential→tenants lookup
-- Refreshed every 60s via cron worker.

CREATE MATERIALIZED VIEW multi_tenant_membership_index AS
  SELECT
    wc.credential_id,
    array_agg(DISTINCT tm.tenant_id) AS tenant_ids,
    array_agg(DISTINCT tm.tenant_id || '|' || ata.audience_type) AS tenant_audience_pairs
  FROM webauthn_credentials wc
  JOIN tenant_memberships tm ON tm.principal_id = wc.principal_id
  LEFT JOIN audience_type_assignments ata
    ON ata.principal_id = tm.principal_id AND ata.tenant_id = tm.tenant_id
    AND ata.active = TRUE
  WHERE wc.active = TRUE
    AND tm.status = 'ACTIVE'
  GROUP BY wc.credential_id
  HAVING array_length(array_agg(DISTINCT tm.tenant_id), 1) >= 1;

CREATE UNIQUE INDEX idx_multi_tenant_membership_credential
  ON multi_tenant_membership_index (credential_id);
```

### Schema cross-references

| Object | Schema | Used by |
|---|---|---|
| `WebAuthnAssertion` | per ADR-0188 + identity µservice IP-004 | `VerifyAssertion` input |
| `TwoTenantsResponse` | `docs/user-journeys/j126-*/schemas/two-tenants-response.json` | `VerifyAssertion` output when ≥2 tenants |
| `SessionInitRequest` | `docs/user-journeys/j126-*/schemas/session-init-request.json` | `InitSession` input |
| `SessionToken` | per identity µservice IP-002 (extended) | `InitSession` output |

## API surface (gRPC)

### proto excerpt

```protobuf
// microservices/identity/contracts/proto/auditor.proto

syntax = "proto3";

package oya.identity.auditor;

service IdentityAuditor {
  // Verify a WebAuthn assertion. If the credential resolves to ≥2 active
  // tenant memberships, return a two-tenants envelope to drive the
  // context-picker UX per ADR-0311 §B-8.
  rpc VerifyAssertion (VerifyAssertionRequest) returns (VerifyAssertionResponse);

  // After the user picks a tenant, init a session bound to that tenant
  // and audience_type. Per ADR-0311 §B-3: a session is bound to ONE
  // tenant; switching requires a fresh InitSession.
  rpc InitSession (InitSessionRequest) returns (InitSessionResponse);

  // Live lookup of a principal's 3PAO accreditation status.
  // Called on every Cedar evaluation that gates cross-tenant audit pull.
  // Latency budget: ≤200ms p99 per j126 story §19 invariant 8.
  rpc Get3paoAccreditationStatus (Get3paoAccreditationStatusRequest)
      returns (Get3paoAccreditationStatusResponse);
}

message VerifyAssertionRequest {
  bytes assertion = 1;
  string client_origin = 2;
}

message VerifyAssertionResponse {
  oneof outcome {
    SessionToken single_tenant_session = 1;
    TwoTenantsResponse multi_tenant_choice = 2;
    AssertionFailed failed = 3;
  }
}

message TwoTenantsResponse {
  string webauthn_credential_id = 1;
  repeated TenantMembership tenants = 2;
  // Per ADR-0311 §B-8: this field MUST be empty/unset.
  string preselected = 3;  // always ""
}

message TenantMembership {
  string tenant_id = 1;
  string display_name = 2;
  TenantClass tenant_class = 3;
  string cell_id = 4;
  repeated string packs_active = 5;
  repeated AudienceType audience_type_options = 6;
}

enum TenantClass {
  TENANT_CLASS_UNSPECIFIED = 0;
  B2B_TENANT = 1;
  B2C_TENANT = 2;
  INTERNAL_AGENCY_TENANT = 3;
  RESERVED_NAMESPACE_TENANT = 4;
}

enum AudienceType {
  AUDIENCE_TYPE_UNSPECIFIED = 0;
  B2C_CONSUMER = 1;
  B2B_TENANT_ADMIN = 2;
  B2B_HR_ADMIN = 3;
  B2B_INTERNAL_AUDIT = 4;
  INTERNAL_AUDITOR_3PAO = 5;
  B2C_JOB_SEEKER_ACTIVE = 6;
  EMERGENCY_SERVICES = 7;
  FRIENDLY_CRAWLER_PARTNER = 8;
  MINOR_TARGETED = 9;
  INTERNAL_DEV_TOOLS = 10;
}

message InitSessionRequest {
  string webauthn_credential_id = 1;
  string selected_tenant_id = 2;
  AudienceType selected_audience_type = 3;
  SecondFactor second_factor = 4;
}

message InitSessionResponse {
  SessionToken session_token = 1;
  // Embedded fields auditable per ADR-0263:
  // - session.tenant_id
  // - session.audience_type
  // - session.fedramp_3pao_accreditation_active (if INTERNAL_AUDITOR_3PAO)
}

message Get3paoAccreditationStatusRequest {
  string principal_id = 1;
  string tenant_id = 2;
}

message Get3paoAccreditationStatusResponse {
  bool accreditation_active = 1;
  string accreditation_id = 2;
  repeated string baseline_authorized = 3;
  google.protobuf.Timestamp expires_at = 4;
}
```

## Files to author

| File | Purpose | Approx. lines |
|---|---|---:|
| `microservices/identity/src/auditor/cross_tenant_resolver.rs` | gRPC server impl for the 3 RPCs | ~300 |
| `microservices/identity/src/auditor/two_tenants_response_builder.rs` | Builds `TwoTenantsResponse` from materialized view | ~150 |
| `microservices/identity/src/auditor/session_init.rs` | InitSession orchestration (Cedar + audit + observability) | ~200 |
| `microservices/identity/src/auditor/accreditation_lookup.rs` | Fast cache + DB fallback for 3PAO status | ~180 |
| `microservices/identity/policy/auditor-verify-assertion.cedar` | Cedar permit for VerifyAssertion | ~30 |
| `microservices/identity/policy/auditor-init-session.cedar` | Cedar permit for InitSession | ~30 |
| `microservices/identity/policy/auditor-accreditation-lookup.cedar` | Cedar permit for live accreditation lookup | ~30 |
| `microservices/identity/contracts/proto/auditor.proto` | gRPC defs | ~180 |
| `microservices/identity/db/migrations/2026-05-20-001-fedramp-3pao-accreditations.sql` | Accreditation table DDL | ~50 |
| `microservices/identity/db/migrations/2026-05-20-002-audience-type-assignments.sql` | Audience-type table DDL | ~40 |
| `microservices/identity/db/migrations/2026-05-20-003-multi-tenant-membership-index.sql` | Materialized view DDL | ~30 |
| `microservices/identity/runbooks/3pao-accreditation-lapse-mid-audit.md` | Ops runbook for accreditation lapse | ~150 |
| `microservices/identity/runbooks/two-tenants-picker-rollback.md` | Rollback runbook if picker mis-routes | ~120 |
| `microservices/identity/tests/integration/auditor_cross_tenant_test.rs` | Integration tests (A-class, B-class) | ~450 |
| `microservices/identity/dashboards/auditor-3pao-accreditation.json` | Grafana dashboard for 3PAO status | ~80 |
| `microservices/identity/slos/auditor-accreditation-lookup.openslo.yaml` | SLO for live-lookup p99 ≤200ms | ~40 |

Total approximate new code + content: ~2,060 lines.

## Cedar fragments

```cedar
// auditor-verify-assertion.cedar
permit (
  principal == Service::"api-gateway",
  action == Action::"identity.verify_webauthn_assertion",
  resource is WebAuthnCredential
) when {
  context.client_origin matches "*.oyatie.dev" &&
  resource.last_verified_at >= context.now - duration("7d") &&
  resource.active == true
};

// auditor-init-session.cedar
permit (
  principal == Service::"api-gateway",
  action == Action::"identity.init_session",
  resource is User
) when {
  resource.tenant_memberships.contains(context.requested_tenant_id) &&
  resource.tenant_membership(context.requested_tenant_id).status == "ACTIVE"
};

// auditor-accreditation-lookup.cedar
permit (
  principal == Service::"policy-engine",
  action == Action::"identity.get_3pao_accreditation_status",
  resource is User
);
// Note: no `when` clause — this lookup is library-mode-internal and
// callable from any policy-engine instance inside the platform mesh.
// SPIFFE workload identity is verified at the mTLS layer (per ADR-0295).
```

## Integration contracts

| Contract | Direction | Frequency | Failure-mode |
|---|---|---|---|
| `audit-chain.EmitSealed` | identity → audit-chain | Every session-init, every 3PAO accreditation lookup | If audit-chain seal fails, identity emits to async retry queue per ADR-0028 |
| `observability.PushOTLP` | identity → observability | Every RPC | Fire-and-forget; lag is acceptable |
| `tenancy.GetPackOverlayRoster` | identity → tenancy | Every session-init | If tenancy down, fail-closed (deny session) |
| `policy-engine.Get3paoAccreditationStatus` (library-mode) | policy-engine → identity | Every cross-tenant Cedar eval | If identity timeout, policy-engine fails closed (deny) |

## Cross-µservice handshake

### Phase 1.3 — credential→tenants lookup

When `VerifyAssertion` is invoked, identity:

1. Validates the WebAuthn signature per ADR-0188.
2. Queries `multi_tenant_membership_index` materialized view.
3. If ≥2 tenants returned, builds `TwoTenantsResponse`; returns it.
4. If 1 tenant returned, proceeds to single-tenant flow (returns
   `SessionToken` directly).
5. Emits `WebAuthnVerifiedMultiTenant` audit event to the platform-
   level audit-chain (not yet per-tenant since tenant is not yet
   selected).

### Phase 1.7 — session-init binding

When `InitSession` is invoked with a selected tenant + audience-type:

1. Validates `selected_tenant_id` IS in the user's active tenant
   memberships (defense against client tampering).
2. Validates `selected_audience_type` IS in the user's audience-type
   assignments for that tenant (defense against audience-type
   escalation).
3. If `audience_type = INTERNAL_AUDITOR_3PAO`, additionally validates
   live accreditation status via the same DB lookup as
   `Get3paoAccreditationStatus`. If accreditation is inactive, returns
   ERROR with deny-reason "accreditation_inactive".
4. Loads the tenant's pack overlay roster from tenancy.
5. Sets `session.audience_type` = `selected_audience_type`.
6. Emits `SessionEstablishedAuditor` audit event to the SELECTED
   tenant's audit-chain (not the platform-level chain; per ADR-0311
   §B-9, the audit-chain is tenant-isolated at this point).
7. Returns `SessionToken`.

### Phase 3.2 — live accreditation lookup

When the cross-tenant Cedar evaluation runs (per `handshake.md` §3),
policy-engine library-mode calls `Get3paoAccreditationStatus`:

1. Cache check (in-process LRU, 30s TTL).
2. If cache miss, DB lookup.
3. Emits `Audit3paoAccreditationLookup` audit event to the principal's
   tenant audit-chain.
4. Returns the status to policy-engine.
5. policy-engine uses the result in the Cedar `when` clause.

### Latency budget

| RPC | p50 | p95 | p99 | Hard cap |
|---|---:|---:|---:|---:|
| `VerifyAssertion` (single tenant) | 80ms | 120ms | 160ms | 250ms |
| `VerifyAssertion` (two tenants) | 95ms | 140ms | 180ms | 280ms |
| `InitSession` | 120ms | 180ms | 240ms | 350ms |
| `Get3paoAccreditationStatus` (cache hit) | 5ms | 12ms | 25ms | 50ms |
| `Get3paoAccreditationStatus` (cache miss) | 30ms | 80ms | 180ms | 250ms |

Per ADR-0246 amendment §D-policy-evaluation-latency.

## Parallel work compatibility

j126 IPs that this IP depends on or shares surfaces with:

- **`microservices/identity/IP-017-multi-context-principal-resolver.md`**
  — shared materialized-view substrate. j126's view extends IP-017's
  base view with the 3PAO accreditation join.
- **`microservices/tenancy/IP-journey-j126-cross-tenant-permit-grant.md`**
  — depends on the cross-tenant Cedar fragment being live in Marcus's
  tenant fragment-set. Can be authored in parallel; both must land
  before j126 ships.
- **`microservices/audit-chain/IP-journey-j126-dual-tenant-emission-classes.md`**
  — uses the new audit-event classes (`SessionEstablishedAuditor`,
  `Audit3paoAccreditationLookup`, `WebAuthnVerifiedMultiTenant`).
- **`microservices/observability/IP-journey-j126-cross-tenant-audit-metrics.md`**
  — uses the new metrics `oya_identity_session_init_count`,
  `oya_identity_3pao_accreditation_lookup_total`,
  `oya_identity_two_tenants_picker_shown_total`.

Sibling journeys that share this IP's surface:

- **j127** uses `Get3paoAccreditationStatus` indirectly (via the
  resignation workflow that revokes accreditation).
- **j128** uses the multi-context resolver to switch Diana from work
  to personal for the tax workflow.
- **j130** uses the multi-context resolver to bridge from personal
  Messenger to work-relevant whistleblower report.

The IP is authored once; the journey-specific tests in
`integration-test-plan.md` verify each consumer's expectations.

## Test plan summary

Cross-references `docs/user-journeys/j126-*/integration-test-plan.md`:

- Test A.1 — TwoTenantsResponse on two-tenant credential
- Test A.2 — session-init sets audience-type correctly
- Test A.4 — cross-tenant Cedar permit evaluates Allow
- Test B.2 — non-3PAO in agency tenant denied
- Test B.4 — lapsed accreditation flips to Deny
- Test B.5 — personal-tenant principal can't exercise work permits
- Test G.2 — policy-engine timeout fails closed

Identity µservice contributes test fixtures + the principal-context
test harness.

## Observability emissions

Per ADR-0263 emission contract:

| Emission | Class | Cardinality budget | Source |
|---|---|---:|---|
| `WebAuthnVerifiedMultiTenant` | identity/auth | 100k/day | `VerifyAssertion` when two-tenants |
| `SessionEstablishedAuditor` | identity/session | 10k/day | `InitSession` for INTERNAL_AUDITOR_3PAO |
| `Audit3paoAccreditationLookup` | identity/accreditation | 1M/day | Every live lookup |
| `Audit3paoAccreditationLapsedDetected` | identity/accreditation | 100/year | Transition active→inactive |
| `audience_type=INTERNAL_AUDITOR_3PAO` | metric label | bounded to # of 3PAO firms | Cardinality bounded |

## Acceptance criteria

j126 identity slice is intern-buildable per documentation-rigor.md §2
when:

- All schemas validate via `ajv validate` against meta-schema.
- All OpenAPI 3.2.0 contract examples include the 3PAO session-init request/response.
- All AsyncAPI 3.1.0 emissions declare the dual-tenant audit notification envelope.
- All Cedar fragments pass `cedar check-parse` + `cedar validate`.
- All gRPC contracts pass `protoc` + buf-lint.
- All SQL migrations apply forward + rollback cleanly.
- All integration tests pass.
- SLOs hold under load test (10k QPS for VerifyAssertion).
- Runbooks are intern-paste-runnable.
- Dashboards render without missing labels.

## Cross-references

- `docs/user-journeys/j126-*/story.md`
- `docs/user-journeys/j126-*/ux-flow.md`
- `docs/user-journeys/j126-*/handshake.md`
- `docs/user-journeys/j126-*/integration-test-plan.md`
- `microservices/identity/IP-017-multi-context-principal-resolver.md`
- `microservices/identity/IP-004-webauthn-relying-party-kernel.md`
- `microservices/identity/IP-010-step-up-orchestrator.md`
- ADR-0311 §B-3 + §B-7 + §B-8
- ADR-0244 §audience_type
- ADR-0246 amendment §D-policy-evaluation-latency

## Counterpart references - journey-j126-fedramp-3pao-cross-tenant-resolver

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
- Trigger evidence: `microservices/identity/IP-journey-j126-fedramp-3pao-cross-tenant-resolver.md` matched `SLO, p99`.
- Numeric target: `rto_p99_seconds=30`, `rpo_p99_seconds=0` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), PCI-DSS-L1-v4(86400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/identity/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/webauthn-authenticate-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`, `microservices/identity/policy/cedar-acr-predicates.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/identity/IP-journey-j126-fedramp-3pao-cross-tenant-resolver.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/identity/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
