---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
impl_plan_id: IP-010-rest-surfaces
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-iac
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness, oya-governance-openapi-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: oya-cloud-iac-iac-*-rest crates (all 5 BCs)

## Intent

Implement `-rest` crates for all 5 BCs: iac-renderer-rest, iac-validator-rest, iac-applier-rest, iac-rollback-rest, iac-registry-rest. OpenAPI-defined surfaces matching `contracts/openapi/cloud-iac.yaml`. Cedar policy enforcement at the rest layer.

## ChangeSet boundary

Five new crates per ADR-0105: one `-rest` per BC. Catalog rows. OpenAPI conformance check.

## Concrete File Targets

| Path | Action |
|---|---|
| `iac/src/crates/oya-cloud-iac-iac-renderer-rest/{Cargo.toml,src/lib.rs,src/routes.rs,src/middleware.rs}` | create |
| `iac/src/crates/oya-cloud-iac-iac-validator-rest/{Cargo.toml,src/lib.rs,src/routes.rs}` | create |
| `iac/src/crates/oya-cloud-iac-iac-applier-rest/{Cargo.toml,src/lib.rs,src/routes.rs}` | create |
| `iac/src/crates/oya-cloud-iac-iac-rollback-rest/{Cargo.toml,src/lib.rs,src/routes.rs}` | create |
| `iac/src/crates/oya-cloud-iac-iac-registry-rest/{Cargo.toml,src/lib.rs,src/routes.rs}` | create |
| `iac/catalog/oya-cloud-iac-iac-*-rest.yaml` | create (5 rows) |

## Code Shape

```rust
// renderer-rest/src/routes.rs
use axum::{Router, routing::{post, get}, extract::{State, Path, Json}};

pub fn router<S>() -> Router<S>
where S: Clone + Send + Sync + 'static + Deps {
    Router::new()
        .route("/microservices/:microservice/render", post(trigger_render))
        .route("/microservices/:microservice/render/:render_id", get(get_render_result))
        .layer(middleware::oidc_auth())
        .layer(middleware::cedar_policy_check())
        .layer(middleware::audit_emit())
}

async fn trigger_render<S: Deps>(
    State(deps): State<S>,
    Path(microservice): Path<String>,
    Json(body): Json<TriggerRenderRequest>,
) -> Result<Json<TriggerRenderResponse>, RestError> {
    // Cedar already checked by middleware; usecase invocation
    let render_id = deps.renderer().trigger(&microservice, &body.sha, &body.pack, body.environment).await?;
    Ok(Json(TriggerRenderResponse { render_id, estimated_completion: ... }))
}
```

## Acceptance Gates

```bash
cargo check --workspace -p oya-cloud-iac-iac-*-rest --all-features
cargo nextest run --workspace -p oya-cloud-iac-iac-*-rest --all-features
# OpenAPI conformance: assert handler set matches contracts/openapi/cloud-iac.yaml
cloud-ci/oya-ci governance gate `openapi-conformance` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
```

## Test Plan

Per PHASE-01 rest class: 1 test per route (happy + auth-fail + scope-mismatch); ≥ 2 cross-route flows; 1 e2e per route. Coverage 85% line / 75% branch.

| Test | Verifies |
|---|---|
| `test_trigger_render_happy` | OIDC pass + Cedar allow → 202 |
| `test_trigger_render_auth_fail` | OIDC missing → 401 |
| `test_trigger_render_scope_mismatch` | X-Microservice ≠ JWT claim → 401 |
| `test_apply_spiffe_required` | non-SPIFFE → 401 |
| `integration_render_then_get_result` | trigger + poll result |

## Halt Conditions

- Route handler bypasses Cedar middleware — refuse.
- OpenAPI contract drift uncaught — fix lane.

## Next IP

[`IP-011-worker-binaries.md`](IP-011-worker-binaries.md)

## References

- ADR-0105.
- `contracts/openapi/cloud-iac.yaml`.
- axum docs — `docs.rs/axum/`.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/proto/cloud-iac.proto`, `iac/IP-010-rest-surfaces.md`.
