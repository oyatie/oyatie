---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agentic-slo-gated-promotion
impl_plan_id: IP-007-slo-engine-rest
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-observability
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, openapi-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: oya-observability-slo-engine-rest

## Intent

REST surface per `microservices/observability/contracts/openapi/slo-engine.yaml` (OpenAPI 3.2). axum-based router; OIDC bearer auth; X-Scope-OrgID enforcement; Cedar policy gate per request.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/observability/src/crates/oya-observability-slo-engine-rest/Cargo.toml` | create |
| `.../src/{lib.rs,router.rs,handlers/*.rs,auth.rs,cedar.rs}` | create |
| `microservices/observability/catalog/oya-observability-slo-engine-rest.yaml` | create |
| `Cargo.toml` (workspace) | update |

## Crate Naming

```
NAME: oya-observability-slo-engine-rest
JUSTIFICATION: microservice=observability; bc=slo-engine; layer=rest per ADR-0105 (presentation/entry-point)
```

## Code Shape

```rust
// src/router.rs
use axum::{Router, routing::{get, post}};

pub fn build_router(deps: AppDeps) -> Router {
    Router::new()
        .route("/api/v1/microservices/:ms/eligibility/:env/:sha", get(handlers::get_eligibility_verdict))
        .route("/api/v1/microservices/:ms/slos", get(handlers::list_openslo))
        .route("/api/v1/microservices/:ms/slos/:sli", get(handlers::get_openslo))
        .route("/api/v1/microservices/:ms/release-pointers/:env", get(handlers::get_release_pointer))
        .route("/api/v1/microservices/:ms/burn-rate/:env/:sli", get(handlers::get_burn_rate))
        .route("/api/v1/validate-openslo", post(handlers::validate_openslo))
        .route("/health", get(handlers::health))
        .route("/ready", get(handlers::ready))
        .layer(middleware::oidc_auth())
        .layer(middleware::x_scope_org_id_enforce())
        .layer(middleware::cedar_policy_gate())
        .with_state(deps)
}
```

## Acceptance Gates

```bash
cargo check -p oya-observability-slo-engine-rest --all-features
cargo nextest run -p oya-observability-slo-engine-rest --all-features
# OpenAPI conformance: schemathesis OR oapi-codegen+pact
buck2 build //:quality-lane-registry-authority-check # lane=openapi-conformance --spec microservices/observability/contracts/openapi/slo-engine.yaml
```

## Test Plan

Per PHASE-01 rest class: 1 test per route (happy + auth-fail + tenant-mismatch) + ≥ 2 cross-route flows + 1 e2e via REST integration test. Coverage 85% line / 75% branch.

| Test | Verifies |
|---|---|
| `test_get_eligibility_verdict_happy` | OIDC OK + tenant-match ⇒ 200 |
| `test_get_eligibility_verdict_auth_fail` | missing OIDC ⇒ 401 |
| `test_get_eligibility_verdict_tenant_mismatch` | X-Scope-OrgID != principal.tenant_id ⇒ 401 |
| `test_get_eligibility_verdict_cedar_deny` | Cedar policy denies cross-tenant ⇒ 403 |
| `test_validate_openslo_schema_anonymous` | anonymous allowed per `public-read.cedar` |
| `test_health` + `test_ready` | probes |
| `integration_full_flow` | author SLO → query verdict → query release-pointer roundtrip |

## Halt Conditions

- Direct adapter import — must go through application/usecase ports
- Any handler missing Cedar gate — refuse


## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/observability/IP-007-slo-engine-rest.md` matched `openapi`; contract files `microservices/observability/contracts/openapi/slo-engine.yaml, microservices/observability/contracts/asyncapi/eligibility-events.yaml, microservices/observability/contracts/proto/slo-engine.proto`; type anchor `crates/oya-cloud-observability-api/src/lib.rs::CloudObservabilityAuditRecord`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/observability/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/observability/IP-007-slo-engine-rest.md` matched `SLO`; anchors `microservices/observability/runbooks/clickhouse-restore.md, crates/oya-cloud-observability-api/src/lib.rs`; type anchor `crates/oya-cloud-observability-api/src/lib.rs::CloudObservabilityAuditRecord`.

## Next IP

[`IP-008-slo-engine-worker.md`](IP-008-slo-engine-worker.md)

## References

- `microservices/observability/contracts/openapi/slo-engine.yaml`
- `microservices/observability/policy/*.cedar`
