---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-005-lane-runtime-usecase-adapter-rest
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, openapi-rest-route-parity]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: oya-governance-lane-runtime-{usecase,adapter,rest,worker,sdk,app}

## Intent

Complete the `lane-runtime` BC: usecase orchestrator + adapter (GitHub Actions matrix invocation + Postgres CRUD) + REST surface + worker + SDK + composition-root binary.

## ChangeSet boundary

6 crates: `-usecase`, `-adapter`, `-rest`, `-worker`, `-sdk`, `-app`. Depends on IP-004 (kernel + domain).

## Concrete File Targets

| Crate | Files |
|---|---|
| `-usecase` | `src/dispatch_orchestrator.rs`, `src/admission_gate_orchestrator.rs` |
| `-adapter` | `src/gha_matrix_client.rs` (GitHub Actions matrix dispatch), `src/postgres_lane_state.rs` |
| `-rest` | `src/handlers/{lanes,lane_runs,admission_verdict}.rs` per `contracts/openapi/governance.yaml` |
| `-worker` | `src/main.rs` (continuous dispatcher; consumes `PullRequestOpened` event) |
| `-sdk` | `src/client.rs` (Rust client matching gRPC surface) |
| `-app` | `src/main.rs` (composition root: wires worker + rest + adapter; reads OpenBao secrets) |

## Code Shape

```rust
// -usecase/src/dispatch_orchestrator.rs
use oya_governance_lane_runtime_kernel::*;
use oya_governance_lane_runtime_domain::scheduling::compute_matrix_fanout;

pub async fn dispatch_for_pr(
    pr: PullRequestContext,
    registry: &dyn LaneRegistry,
    dispatcher: &dyn LaneDispatcher,
) -> Result<Vec<LaneRun>, UsecaseError> {
    let lanes = registry.list().await?;
    let requests = compute_matrix_fanout(&pr, &lanes);
    let mut runs = Vec::with_capacity(requests.len());
    for req in requests {
        runs.push(dispatcher.dispatch(req).await?);
    }
    Ok(runs)
}
```

```rust
// -adapter/src/gha_matrix_client.rs
use oya_governance_lane_runtime_kernel::*;

pub struct GhaMatrixClient { /* ... */ }

#[async_trait::async_trait]
impl LaneDispatcher for GhaMatrixClient {
    async fn dispatch(&self, req: LaneRequest) -> Result<LaneRun, KernelError> {
        // POST to GitHub Actions workflow-dispatch endpoint
        // matrix entry per req
        todo!()
    }
    async fn cancel(&self, run_id: &uuid::Uuid) -> Result<(), KernelError> { todo!() }
}
```

```rust
// -rest/src/handlers/lanes.rs (axum)
use axum::{extract::State, response::Json};
use oya_governance_lane_runtime_api::*;

pub async fn list_lanes(State(s): State<AppState>) -> Result<Json<Vec<LaneSummary>>, ApiError> {
    let lanes = s.registry.list().await?;
    Ok(Json(lanes))
}
```

```rust
// -app/src/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config_from_openbao().await?;
    let registry = PostgresLaneRegistry::connect(&config.postgres_dsn).await?;
    let dispatcher = GhaMatrixClient::new(&config.gha_token);
    let app_state = AppState { registry: Arc::new(registry), dispatcher: Arc::new(dispatcher) };

    let app = build_axum_app(app_state.clone());
    let worker = build_worker(app_state);

    tokio::select! {
        result = serve(app) => result,
        result = worker.run() => result,
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-governance-lane-runtime-{usecase,adapter,rest,worker,sdk,app} --all-features
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace
buck2 build //:quality-lane-registry-authority-check # lane=openapi-rest-route-parity --microservice governance
buck2 build //:quality-lane-registry-authority-check # lane=composition-root-only --crate oya-governance-lane-runtime-app
buck2 build //:quality-lane-registry-authority-check # lane=sdk-kernel-only --crate oya-governance-lane-runtime-sdk
```

## Test Plan

| Test | Verifies |
|---|---|
| `usecase::test_dispatch_orchestrator_fanout` | matrix fanout correct |
| `adapter::test_gha_matrix_client_dispatch_mocked` | GHA API contract |
| `rest::test_route_parity_with_openapi` | every OpenAPI path has a handler |
| `worker::test_pr_opened_event_consumer_idempotent` | re-delivery safe |
| `app::test_composition_root_wires_all_layers` | DI integrity |

Coverage: per-layer per PHASE-01.

## Halt Conditions

- REST handler not matching OpenAPI signature → halt; align.
- Adapter imports usecase or domain → halt; refactor.
- App contains business logic → halt; move to usecase.

## Next IP

[`IP-006-policy-engine-kernel-domain.md`](IP-006-policy-engine-kernel-domain.md)

## References

- IP-004 (kernel + domain).
- `microservices/governance/contracts/openapi/governance.yaml`.
- `microservices/governance/policy/ci-scope.cedar`.
