---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-009-capability-executor-api-and-rest
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-runtime
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness, cedar-fragment-coverage]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: oya-foundry-runtime-capability-executor-{api,rest}

## Intent

Protocol-neutral `-api` crate (typed contracts) + Axum-based `-rest` crate exposing the OpenAPI surface from `contracts/openapi/foundry-runtime.yaml`. Cedar policy enforcement at boundary; OIDC bearer validation; X-Scope-OrgID match; idempotency-key dedup.

## ChangeSet boundary

Two new Rust crates.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-runtime-capability-executor-api/Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/requests.rs` | create (DispatchRequest, CancelRequest, etc.) |
| `.../src/responses.rs` | create |
| `.../src/errors.rs` | create |
| `src/crates/oya-foundry-runtime-capability-executor-rest/Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/dispatch_route.rs` | create |
| `.../src/invocation_routes.rs` | create |
| `.../src/session_routes.rs` | create |
| `.../src/autonomy_ceiling_route.rs` | create |
| `.../src/capability_descriptor_route.rs` | create |
| `.../src/validate_descriptor_route.rs` | create (anonymous-allowed) |
| `.../src/health_routes.rs` | create |
| `.../src/middleware/oidc.rs` | create |
| `.../src/middleware/cedar.rs` | create |
| `.../src/middleware/tenant_scope.rs` | create |
| `.../src/middleware/idempotency.rs` | create |

## Crate Naming

```
NAME: oya-foundry-runtime-capability-executor-{api,rest}
```

## Code Shape

```rust
// rest/src/dispatch_route.rs
use axum::{extract::*, http::StatusCode, Json};
use oya_foundry_runtime_capability_executor_api::*;
use oya_foundry_runtime_capability_executor_usecase::DispatchUseCase;

pub async fn dispatch(
    State(app): State<AppState>,
    Path(capability_id): Path<String>,
    Extension(principal): Extension<TenantPrincipal>,  // from OIDC middleware
    Extension(idempotency_key): Extension<Option<String>>,
    Json(req): Json<DispatchRequest>,
) -> Result<(StatusCode, Json<DispatchResponse>), DispatchHttpError> {
    // Cedar policy already evaluated by middleware
    // Idempotency middleware already checked
    let invocation = app.dispatch_use_case.run(
        &principal.tenant_id,
        &capability_id,
        req.input,
        AutonomyTier::from_int(req.autonomy_tier_declared)?,
    ).await?;
    Ok((StatusCode::ACCEPTED, Json(DispatchResponse::from(invocation))))
}

// middleware/cedar.rs
pub async fn cedar_authorize<B>(
    State(authz): State<CedarAuthorizer>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, AuthzError> {
    let principal = req.extensions().get::<TenantPrincipal>().ok_or(AuthzError::Unauthenticated)?;
    let action = req_to_action(&req)?;
    let resource = req_to_resource(&req)?;
    match authz.evaluate(principal, &action, &resource).await? {
        CedarDecision::Allow => Ok(next.run(req).await),
        CedarDecision::Deny => Err(AuthzError::Forbidden),
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-runtime-capability-executor-{api,rest}
cargo nextest run -p oya-foundry-runtime-capability-executor-rest --features integration-tests
cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage --microservice foundry-runtime
cargo run -p oya-dev-cli -- gate validate openapi-conformance --microservice foundry-runtime
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_dispatch_route_happy_path` | 202 + Invocation body |
| `test_dispatch_oidc_missing_returns_401` | OIDC middleware |
| `test_dispatch_tenant_scope_mismatch_returns_401` | X-Scope-OrgID enforcement |
| `test_dispatch_autonomy_violation_returns_403` | Cedar + AutonomyGate refusal |
| `test_dispatch_idempotency_key_dedup` | re-running same key returns prior response |
| `test_dispatch_429_on_rate_limit` | per-tenant rate limit |
| `test_cancel_invocation_route` | 202 then invocation reaches Cancelled state |
| `test_get_session_route_cross_tenant_forbidden` | TI-04 |
| `test_validate_descriptor_anonymous_allowed` | public-read.cedar |

## Halt Conditions

- Any route bypasses Cedar middleware — refactor.
- Idempotency check absent on dispatch — refactor (RFC 7231-compatible behaviour required).

## Next IP

[`IP-010-capability-executor-sdk.md`](IP-010-capability-executor-sdk.md)

## References

- `contracts/openapi/foundry-runtime.yaml`.
- `policy/{tenant-scope,ci-scope,auditor-scope,public-read}.cedar`.
- ADR-0140 (retired per ADR-0145) (Cedar policy enforcement).
