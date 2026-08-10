---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j126-cross-tenant-permit-grant
journey_id: j126-government-auditor-3pao-conducts-fedramp-audit
microservice: tenancy
role: cross-tenant-permit-grant
status: draft
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0294-cedar-fragment-soak
  - ADR-0251-compliance-pack-cell-certification-levels
depends_on:
  - microservices/identity/IP-journey-j126-fedramp-3pao-cross-tenant-resolver.md
  - microservices/audit-chain/IP-journey-j126-dual-tenant-emission-classes.md
date: 2026-05-20
owner_team: axis-tenancy + axis-compliance
parallel_work_compatibility: |
  Independent of j127 (offboarding) work — j127 touches
  tenant-membership revocation. j128 reuses the same cross-tenant
  permit grammar in reverse (Diana's personal-tenant invoices
  agency-tenant tax authority). All four can be authored in parallel.
---

# IP-journey-j126-cross-tenant-permit-grant — Tenancy µservice: cross-tenant Cedar permit grant grammar and FedRAMP authorization workflow

## Goal

Implement tenancy µservice surfaces that grant cross-tenant Cedar
permits — specifically, the FedRAMP 3PAO permit that allows Diana's
agency tenant (`gao.audit.fedramp-3pao`) to read audit evidence in
Marcus's contractor tenant (`chen-aerospace.federal-contractor.us`).

Three surfaces:

1. **`GrantCrossTenantPermit`** — when a CSP enrolls in FedRAMP
   authorization, the FedRAMP PMO authorizes a 3PAO tenant to read
   their audit evidence; this grant emits the Cedar fragment into the
   CSP tenant's fragment-set.
2. **`ListCrossTenantPermitsGranted`** — both grantor (CSP) and grantee
   (3PAO) can list their cross-tenant relationships.
3. **`RevokeCrossTenantPermit`** — emergency revocation if the
   authorization is withdrawn (e.g., 3PAO accreditation lapses, or
   the CSP exits FedRAMP).

These surfaces preserve the **explicit, scoped, attested, soak-windowed,
audited** invariants of ADR-0311 §B-4 cross-tenant permit grammar.

## Data model

### Postgres tables

```sql
-- Migration: 2026-05-20-001-cross-tenant-permits.sql

CREATE TABLE cross_tenant_permits (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  grantor_tenant_id TEXT NOT NULL,
  grantee_tenant_id TEXT NOT NULL,
  permit_class TEXT NOT NULL CHECK (
    permit_class IN (
      'fedramp-3pao-audit',
      'pci-qsa-audit',
      'hipaa-business-associate',
      'soc2-cpa-audit',
      'iso-27001-audit',
      'gdpr-art-28-controller-processor',
      'court-warrant-scope-bounded',     -- ADR-0312
      'b2b-tenant-invited-collaborator',
      'staffing-agency-facilitator'
    )
  ),
  cedar_fragment_id TEXT NOT NULL,        -- references cedar_fragments table
  actions_permitted TEXT[] NOT NULL,      -- e.g., ['audit_chain.read_sealed_evidence', ...]
  effective_at TIMESTAMPTZ NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  granted_by_principal_id TEXT NOT NULL,
  granted_under_authority TEXT NOT NULL,  -- e.g., 'FedRAMP-PMO-2024-10-CSP-CHEN-AERO'
  granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  soak_started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  soak_window_seconds INT NOT NULL DEFAULT 60,  -- ADR-0294 floor
  active BOOLEAN NOT NULL DEFAULT FALSE,
  revoked_at TIMESTAMPTZ,
  revocation_reason TEXT,
  CHECK (expires_at > effective_at),
  CHECK (effective_at >= granted_at + (soak_window_seconds || ' seconds')::INTERVAL)
);

CREATE INDEX idx_cross_tenant_permits_grantor
  ON cross_tenant_permits (grantor_tenant_id) WHERE active = TRUE;

CREATE INDEX idx_cross_tenant_permits_grantee
  ON cross_tenant_permits (grantee_tenant_id) WHERE active = TRUE;

CREATE INDEX idx_cross_tenant_permits_active
  ON cross_tenant_permits (active, effective_at, expires_at);
```

### Schema cross-references

| Object | Schema | Used by |
|---|---|---|
| `CrossTenantPermitGrantRequest` | `schemas/cross-tenant-permit-grant-request.json` (new spec) | `GrantCrossTenantPermit` input |
| `CrossTenantPermitRecord` | `schemas/cross-tenant-permit-record.json` (new spec) | `ListCrossTenantPermitsGranted` output |
| Cedar fragment | per `microservices/policy-engine/contracts/cedar-fragment-schema.json` | Stored fragment text |

## API surface (gRPC)

```protobuf
// microservices/tenancy/contracts/proto/cross_tenant_permit.proto

syntax = "proto3";

package oya.tenancy.cross_tenant;

service TenancyCrossTenantPermit {
  rpc GrantCrossTenantPermit (GrantCrossTenantPermitRequest)
      returns (GrantCrossTenantPermitResponse);
  rpc ListCrossTenantPermitsGranted (ListCrossTenantPermitsGrantedRequest)
      returns (ListCrossTenantPermitsGrantedResponse);
  rpc RevokeCrossTenantPermit (RevokeCrossTenantPermitRequest)
      returns (RevokeCrossTenantPermitResponse);
}

message GrantCrossTenantPermitRequest {
  string grantor_tenant_id = 1;
  string grantee_tenant_id = 2;
  PermitClass permit_class = 3;
  string cedar_fragment_text = 4;
  repeated string actions_permitted = 5;
  google.protobuf.Timestamp effective_at = 6;
  google.protobuf.Timestamp expires_at = 7;
  string granted_under_authority = 8;
  // Per ADR-0294: soak window MUST be ≥60s
  int32 soak_window_seconds = 9;
}

message GrantCrossTenantPermitResponse {
  string permit_id = 1;
  google.protobuf.Timestamp will_be_active_at = 2;  // = effective_at, post-soak
  string cedar_fragment_id = 3;
}

enum PermitClass {
  PERMIT_CLASS_UNSPECIFIED = 0;
  FEDRAMP_3PAO_AUDIT = 1;
  PCI_QSA_AUDIT = 2;
  HIPAA_BUSINESS_ASSOCIATE = 3;
  SOC2_CPA_AUDIT = 4;
  ISO_27001_AUDIT = 5;
  GDPR_ART_28_CONTROLLER_PROCESSOR = 6;
  COURT_WARRANT_SCOPE_BOUNDED = 7;       // ADR-0312
  B2B_TENANT_INVITED_COLLABORATOR = 8;
  STAFFING_AGENCY_FACILITATOR = 9;
}

message ListCrossTenantPermitsGrantedRequest {
  oneof scope {
    string as_grantor_tenant_id = 1;
    string as_grantee_tenant_id = 2;
  }
  bool include_inactive = 3;
}

message ListCrossTenantPermitsGrantedResponse {
  repeated CrossTenantPermitRecord permits = 1;
}

message CrossTenantPermitRecord {
  string permit_id = 1;
  string grantor_tenant_id = 2;
  string grantee_tenant_id = 3;
  PermitClass permit_class = 4;
  repeated string actions_permitted = 5;
  google.protobuf.Timestamp effective_at = 6;
  google.protobuf.Timestamp expires_at = 7;
  string granted_under_authority = 8;
  bool active = 9;
  string cedar_fragment_id = 10;
}

message RevokeCrossTenantPermitRequest {
  string permit_id = 1;
  string revocation_reason = 2;
  // Emergency revocation: bypasses normal soak; cuts effect immediately.
  bool emergency = 3;
}

message RevokeCrossTenantPermitResponse {
  google.protobuf.Timestamp revoked_at = 1;
}
```

## Files to author

| File | Purpose | Approx. lines |
|---|---|---:|
| `microservices/tenancy/src/cross_tenant/grant.rs` | gRPC server impl + soak orchestration | ~280 |
| `microservices/tenancy/src/cross_tenant/list.rs` | List endpoint impl | ~120 |
| `microservices/tenancy/src/cross_tenant/revoke.rs` | Revocation impl | ~180 |
| `microservices/tenancy/src/cross_tenant/cedar_fragment_publisher.rs` | Publishes fragment to policy-engine fragment-store with soak | ~200 |
| `microservices/tenancy/policy/grant-cross-tenant-permit.cedar` | Cedar permit for granting (only platform PMO authority) | ~30 |
| `microservices/tenancy/policy/list-cross-tenant-permits.cedar` | Cedar permit for listing | ~30 |
| `microservices/tenancy/policy/revoke-cross-tenant-permit.cedar` | Cedar permit for revoking | ~30 |
| `microservices/tenancy/contracts/proto/cross_tenant_permit.proto` | gRPC defs | ~150 |
| `microservices/tenancy/db/migrations/2026-05-20-001-cross-tenant-permits.sql` | DDL | ~50 |
| `microservices/tenancy/runbooks/cross-tenant-permit-emergency-revocation.md` | Runbook for emergency revoke | ~140 |
| `microservices/tenancy/runbooks/fedramp-3pao-authorization-onboarding.md` | Onboarding runbook for new CSP | ~180 |
| `microservices/tenancy/tests/integration/cross_tenant_permit_test.rs` | Integration tests | ~400 |
| `microservices/tenancy/dashboards/cross-tenant-permits.json` | Grafana for cross-tenant permit status | ~80 |
| `microservices/tenancy/slos/cross-tenant-permit-grant.openslo.yaml` | SLO for grant latency | ~40 |
| `microservices/tenancy/schemas/cross-tenant-permit-grant-request.json` | Spec | ~80 |
| `microservices/tenancy/schemas/cross-tenant-permit-record.json` | Spec | ~80 |

Total approximate new code + content: ~2,070 lines.

## Cedar fragments

```cedar
// grant-cross-tenant-permit.cedar
// ONLY the platform PMO (FedRAMP authorization office) can grant
// FedRAMP cross-tenant permits. ADR-0311 §B-4 attestation invariant.

permit (
  principal == User::"fedramp-pmo-authorizing-official",
  action == Action::"tenancy.grant_cross_tenant_permit",
  resource is Tenant
) when {
  context.permit_class == "fedramp-3pao-audit" &&
  context.grantor_tenant_authorization_status == "FedRAMP-authorized" &&
  context.grantee_tenant_3pao_accreditation_active == true
};

// list-cross-tenant-permits.cedar
permit (
  principal is User,
  action == Action::"tenancy.list_cross_tenant_permits",
  resource is Tenant
) when {
  // Both grantor's tenant admins AND grantee's tenant admins can list
  // permits that touch their tenant
  principal.tenant == resource.id ||
  principal.audience_type == "B2B_TENANT_ADMIN" ||
  principal.audience_type == "INTERNAL_AUDITOR_3PAO"
};

// revoke-cross-tenant-permit.cedar
permit (
  principal is User,
  action == Action::"tenancy.revoke_cross_tenant_permit",
  resource is CrossTenantPermit
) when {
  // EITHER the grantor's tenant admin OR the issuing authority can revoke
  principal.tenant == resource.grantor_tenant_id ||
  principal.id == resource.granted_by_principal_id ||
  principal.audience_type == "FEDRAMP_PMO_OFFICIAL"
};
```

## Integration contracts

| Contract | Direction | Frequency | Failure-mode |
|---|---|---|---|
| `policy-engine.PublishFragment` | tenancy → policy-engine | Every grant | If publish fails, grant is rolled back |
| `audit-chain.EmitSealed` | tenancy → audit-chain (both tenants) | Every grant + revoke | Async retry per ADR-0028 |
| `comms-email.SendTenantAdminNotification` | tenancy → comms-email | Every grant | Async; retry queue |
| `compliance.RegisterCrossTenantAttestation` | tenancy → compliance | Every fedramp-class grant | If compliance fails, grant warns but proceeds |

## Cross-µservice handshake

### Grant flow (when FedRAMP PMO authorizes Marcus's tenant for Diana's audit)

```
FedRAMP PMO       api-gateway      tenancy        policy-engine    audit-chain (Marcus)    audit-chain (GAO)
   │                  │              │                  │                  │                      │
   │ grant request    │              │                  │                  │                      │
   ├─────────────────►│              │                  │                  │                      │
   │                  │ Cedar permit │                  │                  │                      │
   │                  ├─────────────►│                  │                  │                      │
   │                  │              │ validate input   │                  │                      │
   │                  │              │ INSERT permit    │                  │                      │
   │                  │              │   (active=FALSE) │                  │                      │
   │                  │              │ publish fragment │                  │                      │
   │                  │              ├─────────────────►│                  │                      │
   │                  │              │                  │ soak ≥60s        │                      │
   │                  │              │ emit audit       │                  │                      │
   │                  │              ├─────────────────────────────────────►│                      │
   │                  │              │ emit audit       │                  │                      │
   │                  │              ├──────────────────────────────────────────────────────────►│
   │                  │              │ (60s later)      │                  │                      │
   │                  │              │ UPDATE active=T  │                  │                      │
   │                  │              │ emit audit       │                  │                      │
   │                  │              ├─────────────────────────────────────►│ (post-soak active)   │
   │                  │              ├──────────────────────────────────────────────────────────►│
```

### Soak window — ADR-0294 enforcement

The 60-second soak window is enforced TWO ways:

1. **DB check constraint**: `effective_at >= granted_at + soak_window_seconds`. Inserting a row with smaller delta fails at DDL.
2. **policy-engine fragment store**: even if `cross_tenant_permits.active = TRUE` is set in DB, policy-engine evaluates fragments only when their own soak window has elapsed in the fragment store.

This is **belt-and-suspenders** per documentation-rigor.md §3.2.6.D
prevention invariant 1 (no single point of failure).

### Latency budget

| RPC | p50 | p95 | p99 | Hard cap |
|---|---:|---:|---:|---:|
| `GrantCrossTenantPermit` | 180ms | 320ms | 480ms | 800ms |
| `ListCrossTenantPermitsGranted` | 40ms | 90ms | 140ms | 250ms |
| `RevokeCrossTenantPermit` (normal) | 220ms | 380ms | 540ms | 900ms |
| `RevokeCrossTenantPermit` (emergency) | 80ms | 140ms | 200ms | 350ms |

## Parallel work compatibility

This IP is **the** authoritative cross-tenant permit grant. Sibling
journeys consume it:

- **j127 (resignation)**: uses `RevokeCrossTenantPermit` to revoke
  staffing-agency cross-tenant permits when the engineer departs.
- **j128 (Diana's tax workflow)**: uses cross-tenant permit grammar
  in reverse — Diana's personal-tenant invoices IRS tenant via the
  same grant mechanism.
- **j129 (court warrant)**: uses `COURT_WARRANT_SCOPE_BOUNDED` permit
  class (separate ADR-0312 grammar; same primitive).
- **j130 (bribery report)**: uses `LIST_CROSS_TENANT_PERMITS` to
  discover that Diana's personal-tenant has zero cross-tenant
  permits to GAO (the architecture's hard guarantee).
- **j131 (cross-jurisdiction audit)**: extends `FEDRAMP_3PAO_AUDIT`
  with EU + KR per-jurisdiction overlays.

Independent µservice IPs co-authored:
- identity µservice (this same docket — see j126 identity IP)
- audit-chain µservice (see j126 audit-chain IP)
- compliance µservice (see j126 compliance IP)
- ops-dashboard µservice (see j126 ops-dashboard IP)
- observability µservice (see j126 observability IP)

## Test plan summary

Cross-references `docs/user-journeys/j126-*/integration-test-plan.md`:

- Test A.4 — cross-tenant Cedar permit evaluates Allow
- Test B.3 — expired permit denied
- Test B.4 — lapsed accreditation forces revocation
- Test B.6 — no permit exists for agency→personal-tenant read

Tenancy µservice contributes test fixtures: synthetic FedRAMP grant
fixtures + permit-expiry fixtures + revocation fixtures.

## Observability emissions

Per ADR-0263:

| Emission | Class | Cardinality | Source |
|---|---|---:|---|
| `CrossTenantPermitGranted` | tenancy/cross-tenant | ~1000/year | `GrantCrossTenantPermit` |
| `CrossTenantPermitActive` | tenancy/cross-tenant | ~1000/year | Post-soak transition |
| `CrossTenantPermitRevoked` | tenancy/cross-tenant | ~50/year | `RevokeCrossTenantPermit` |
| `CrossTenantPermitExpired` | tenancy/cross-tenant | ~500/year | Cron emits on expiry |

## Acceptance criteria

j126 tenancy slice is intern-buildable when:

- All schemas validate.
- All Cedar fragments pass parse/validate.
- DB migrations apply forward + rollback.
- Soak window enforced (test: grant with `soak_window_seconds=30` fails).
- Integration tests pass.
- SLOs hold under 1000 QPS for `ListCrossTenantPermitsGranted`.

## Cross-references

- `docs/user-journeys/j126-*/story.md`
- `docs/user-journeys/j126-*/handshake.md`
- `docs/user-journeys/j126-*/integration-test-plan.md`
- ADR-0311 §B-4 cross-tenant permit grammar
- ADR-0294 Cedar fragment soak
- ADR-0312 court-warrant-scoped piercing (sibling permit class)

## Completion expansion — j126 tenancy IP rigor pass

Journey context: FedRAMP 3PAO audit with Diana work/personal tenant separation.
Service role: tenant membership, sub-scope, residency, and cross-tenant grant boundary.
Mapped services in this journey: identity, tenancy, audit-chain, compliance, ops-dashboard-control-center, observability.
ADR anchors: ADR-0244, ADR-0299, ADR-0311, ADR-0312, ADR-0313, ADR-0314, ADR-0315, ADR-0316, ADR-0317, ADR-0318, ADR-0319, ADR-0320.
This IP is sized as a single reviewable implementation slice and remains compatible with the 56-µservice flat layout.

Implementation task 001: in tenancy, define the Cedar policy change for FedRAMP 3PAO audit with Diana work/personal tenant separation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 001: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 001: add property coverage proving tenancy and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 001: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 002: in tenancy, define the OpenAPI 3.2.0 contract change for FedRAMP 3PAO audit with Diana work/personal tenant separation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 002: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 002: add contract coverage proving tenancy and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 002: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 003: in tenancy, define the AsyncAPI 3.1.0 event change for FedRAMP 3PAO audit with Diana work/personal tenant separation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 003: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 003: add integration coverage proving tenancy and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 003: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 004: in tenancy, define the proto3 port change for FedRAMP 3PAO audit with Diana work/personal tenant separation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 004: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 004: add replay coverage proving tenancy and ops-dashboard-control-center agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 004: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 005: in tenancy, define the Postgres/RLS storage change for FedRAMP 3PAO audit with Diana work/personal tenant separation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 005: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0314 scope grants it; refusals are success states, not exceptions.
Verification 005: add load coverage proving tenancy and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 005: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 006: in tenancy, define the audit-chain emission change for FedRAMP 3PAO audit with Diana work/personal tenant separation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 006: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0315 scope grants it; refusals are success states, not exceptions.
Verification 006: add chaos coverage proving tenancy and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 006: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 007: in tenancy, define the dashboard projection change for FedRAMP 3PAO audit with Diana work/personal tenant separation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 007: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0316 scope grants it; refusals are success states, not exceptions.
Verification 007: add negative authorization coverage proving tenancy and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 007: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 008: in tenancy, define the runbook hook change for FedRAMP 3PAO audit with Diana work/personal tenant separation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 008: tenancy MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 008: add multi-region coverage proving tenancy and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 008: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `tenancy/IP-journey-j126-cross-tenant-permit-grant.md` matched `.proto`; contract files `tenancy/contracts/openapi/tenancy.yaml, tenancy/contracts/asyncapi/tenant-events.yaml, tenancy/contracts/proto/tenancy.proto`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.

## DR posture (per ADR-0343)
- Manifest target source: `tenancy/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `tenancy/IP-journey-j126-cross-tenant-permit-grant.md` matched `p99, SLO, multi-region`; anchors `tenancy/runbooks/dr-pair-promotion-drill.md, crates/oya-tenancy-api/src/lib.rs`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `tenancy/IP-journey-j126-cross-tenant-permit-grant.md` matched `emission`; anchors `tenancy/manifest.json, crates/oya-tenancy-api/src/lib.rs`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.
