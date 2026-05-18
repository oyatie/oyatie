---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-011-rest-and-grpc-surface
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, openapi-conformance, grpc-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: REST + gRPC surface (one `-rest` crate per BC; consolidated OpenAPI + proto)

## Intent

Author `-rest` crates for all 6 BCs (prompt-classifier, output-validator, autonomy-tier-gate, content-safety-rule-engine, jailbreak-detector, ai-slop-detector). OpenAPI 3.2 + gRPC contracts at `contracts/openapi/guardrails.yaml` + `contracts/proto/guardrails.proto`. Per-route Cedar policy bind. Tenant tracing via X-Scope-OrgID. `-api` crates introduced per BC to carry protocol-neutral request/response types.

## ChangeSet boundary

Each BC gets a `-rest` + `-api` crate. Consolidated OpenAPI + proto cover all BC actions. Composition for runtime is wired in IP-012.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-guardrails-<bc>-api/Cargo.toml` + `src/{lib.rs,requests.rs,responses.rs,errors.rs}` (× 6 BCs) | create |
| `src/crates/oya-foundry-guardrails-<bc>-rest/Cargo.toml` + `src/{lib.rs,routes.rs,middleware.rs}` (× 6 BCs) | create |
| `contracts/openapi/guardrails.yaml` | already created above; populate paths per BC |
| `contracts/proto/guardrails.proto` | already created above; populate services per BC |
| `contracts/asyncapi/decision-events.yaml` | already created above |
| `Cargo.toml` workspace | update |
| `catalog/oya-foundry-guardrails-<bc>-{api,rest}.yaml` | create (12 catalog rows) |

## Code Shape

```rust
// prompt-classifier-rest/src/routes.rs
use axum::{Router, routing::post};

pub fn router<S: AppState>() -> Router<S> {
    Router::new()
        .route("/v1/classify-prompt", post(classify_prompt))
        .route("/v1/validate-output", post(validate_output))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tenant_scope_middleware)
        .layer(cedar_policy_middleware)
}

async fn classify_prompt(
    State(state): State<S>,
    cedar_decision: CedarDecision,    // injected by middleware
    Json(req): Json<ClassifyPromptRequest>,
) -> Result<Json<ClassifyPromptResponse>, ApiError> {
    cedar_decision.require_allow()?;
    let classification = state.use_case().classify(&req.into_prompt()).await?;
    Ok(Json(classification.into()))
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-guardrails-<bc>-rest (× 6) --all-features
cargo nextest run -p oya-foundry-guardrails-<bc>-rest (× 6) --all-features
cargo run -p oya-dev-cli -- gate validate openapi-conformance --microservice foundry-guardrails
cargo run -p oya-dev-cli -- gate validate grpc-conformance --microservice foundry-guardrails
```

## Test Plan

Per rest class: 1 test per route (happy + auth-fail + tenant-mismatch) + ≥ 2 cross-route flows + 1 e2e per route. Coverage 85% / 75%.

## Halt Conditions

- Any route without Cedar middleware — refuse merge.
- OpenAPI / proto drift — refuse merge.

## Next IP

[`IP-012-worker-and-app-composition.md`](IP-012-worker-and-app-composition.md)

## References

- ADR-0056, ADR-0105, ADR-0140 (retired per ADR-0145).
- `contracts/openapi/guardrails.yaml`.
- `contracts/proto/guardrails.proto`.
- `policy/tenant-scope.cedar`.
