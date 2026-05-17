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
cargo run -p oya-dev-cli -- gate validate openapi-conformance --spec microservices/observability/contracts/openapi/slo-engine.yaml
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

## Next IP

[`IP-008-slo-engine-worker.md`](IP-008-slo-engine-worker.md)

## References

- `microservices/observability/contracts/openapi/slo-engine.yaml`
- `microservices/observability/policy/*.cedar`
