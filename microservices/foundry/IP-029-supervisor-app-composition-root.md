---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-014-app-composition-root
status: pending
execution_unit: ChangeSet
owner: axis-foundry-control-plane
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, lean-a2]
depends_on: [IP-001, IP-002, IP-003, IP-004, IP-005, IP-006, IP-007, IP-008, IP-009, IP-010, IP-011, IP-012, IP-013]
---

# IP-014: app composition root (binaries wiring all BCs)

## Intent

Composition-root binaries per BC. Wires kernel/domain/usecase via ports to concrete adapters; mTLS + SPIFFE; OpenBao SecretReference; lease-leadership election.

## Concrete File Targets

`microservices/foundry/src/crates/oya-foundry-supervisor-{bc}-app/`:
- agent-fleet-lifecycle-app
- capability-deployment-app
- autonomy-policy-enforcement-app
- supervision-event-bus-app
- kill-switch-circuit-breaker-app

Each has `[bin] main = "src/main.rs"` + minimal wiring code.

## Key code

```rust
// kill-switch-circuit-breaker-app/src/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // SPIFFE identity bootstrap
    let spiffe_identity = SpiffeIdentity::from_env()?;

    // OpenBao secret materialisation
    let openbao = OpenbaoClient::connect_with_spiffe(&spiffe_identity).await?;
    let signing_key = openbao.fetch_ed25519_signing_key("supervisor/kill-switch").await?;

    // Postgres pool
    let postgres_pool = build_postgres_pool(&openbao).await?;

    // Redis cluster
    let redis = build_redis_cluster_client(&openbao).await?;

    // Kubernetes client
    let k8s = kube::Client::try_default().await?;

    // Wire ports → adapters
    let state_store: Arc<dyn KillSwitchStateStore> = Arc::new(RedisKillSwitchStateStore::new(redis.clone(), signing_key.clone()));
    let propagator: Arc<dyn KillSwitchPropagator> = Arc::new(KubernetesCrdPropagator::new(k8s.clone()));
    let publisher: Arc<dyn SupervisionEventPublisher> = Arc::new(RedisSupervisionEventPublisher::new(redis.clone(), signing_key.clone()));
    let cedar: Arc<dyn CedarEvaluator> = Arc::new(CedarV4Evaluator::load_policy("policy/")?);

    // Wire REST surface
    let rest_app = build_rest_app(state_store.clone(), propagator.clone(), publisher.clone(), cedar.clone());

    // Wire worker (CRD watch + Redis pub-sub fan-out)
    let worker = KillSwitchWorker::new(state_store.clone(), propagator.clone(), publisher.clone());

    // Lease-leadership election for worker
    let lease = kube::api::Api::namespaced(k8s.clone(), &spiffe_identity.namespace)
        .lease("foundry-supervisor-kill-switch-leader").await?;

    // Start REST + (worker behind leadership lease)
    tokio::try_join!(
        rest_app.serve(),
        worker.run_with_leadership_lease(lease),
    )?;

    Ok(())
}
```

## Acceptance Gates

```bash
cargo check / build / clippy / nextest per app crate
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry-supervisor
```

## Test Plan

| Test | Verifies |
|---|---|
| Startup smoke (composition root brings up cleanly) | composition-root wiring valid |
| Shutdown smoke (SIGTERM gracefully completes in-flight) | graceful shutdown |
| Lease leadership election (two replicas, one elected) | HA |
| OpenBao SecretReference materialisation | no raw secrets in binaries |

## Halt Conditions

- App imports `usecase`-internal types directly (use ports).
- App contains business logic.

## Next IP

[`IP-015-e2e-drills-and-dashboards.md`](IP-015-e2e-drills-and-dashboards.md)

## References

- ADR-0105 (`app` layer).
- ADR-0056 BNF v4.1.
- PRD §"Bounded Contexts".
- kube-rs lease leadership — `docs.rs/kube`.
