---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-011-capability-executor-app
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-runtime
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
---

# IP-011: oya-foundry-runtime-capability-executor-app (composition root)

## Intent

Composition root binary wiring all five BCs' app crates: capability-executor + invocation-orchestrator + runtime-pool + capability-registry-cache + session-state. Owns the binary boot sequence: load config → bind OpenBao secrets → connect Redis + Postgres → register Cedar policy fragments → start REST/gRPC servers → start workers → register self-observability.

## ChangeSet boundary

One new Rust crate.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-runtime-capability-executor-app/Cargo.toml` | create |
| `.../src/main.rs` | create |
| `.../src/config.rs` | create |
| `.../src/boot.rs` | create |
| `.../src/wiring.rs` | create (DI graph) |
| `.../src/observability.rs` | create (OTel + Prometheus emission) |
| `.../config/default.toml` | create |

## Crate Naming

```
NAME: oya-foundry-runtime-capability-executor-app
JUSTIFICATION:
- microservice = foundry-runtime; bc-tokens = capability-executor
- layer = app per ADR-0105 (composition root)
- exemptions claimed: none — wires multiple BCs at composition root per Amendment 2
  (app crates may compose multiple BCs within the same µservice)
```

## Code Shape

```rust
// src/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    let observability = init_observability(&config)?;

    let redis = connect_redis(&config.redis).await?;
    let postgres = connect_postgres(&config.postgres).await?;
    let openbao = connect_openbao(&config.openbao).await?;
    let cedar = load_cedar_policies(&config.policy_dir).await?;

    let session_store = RedisSessionStore::new(redis.clone(), openbao.clone());
    let session_log = PostgresSessionMutationLog::new(postgres.clone());
    let registry_mirror = PostgresRegistryMirror::new(postgres.clone(), supervisor_pubkey(&openbao).await?);
    let cache = InMemoryCacheStore::new();
    let autonomy = AutonomyGateAdapter::new(postgres.clone());

    let provider_invoker = ProviderInvokerMtls::new(&config.providers_endpoint);
    let guardrail_checker = GuardrailCheckerMtls::new(&config.guardrails_endpoint);
    let evidence_emitter = EvidenceEmitterMtls::new(&config.evidence_endpoint);

    let dispatch_use_case = DispatchUseCase::new(autonomy, cache.clone(), guardrail_checker, provider_invoker, evidence_emitter.clone());

    let app_state = AppState { dispatch_use_case, cedar, ... };

    // Start REST server
    let rest_server = build_rest_app(app_state.clone());
    let rest_handle = tokio::spawn(rest_server.serve(config.rest_bind));

    // Start gRPC server
    let grpc_server = build_grpc_app(app_state.clone());
    let grpc_handle = tokio::spawn(grpc_server.serve(config.grpc_bind));

    // Start workers
    let hot_reload_worker = HotReloadWorker::new(registry_mirror, cache);
    let pool_worker = PoolHealthWorker::new(...);
    let orchestrator_worker = TimeoutMonitorWorker::new(...);
    let session_worker = DsrCascadeWorker::new(...);

    tokio::try_join!(
        rest_handle,
        grpc_handle,
        hot_reload_worker.run(),
        pool_worker.run(),
        orchestrator_worker.run(),
        session_worker.run(),
        watch_for_shutdown(),
    )?;
    Ok(())
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-runtime-capability-executor-app
cargo build -p oya-foundry-runtime-capability-executor-app --release
cargo nextest run -p oya-foundry-runtime-capability-executor-app --test smoke
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry-runtime
```

## Test Plan

Per PHASE-01 app class: composition-root smoke + 1 startup-and-shutdown smoke + 60% coverage (mostly wiring).

| Test | Verifies |
|---|---|
| `test_app_boots_with_default_config` | binary starts; REST + gRPC reachable on /health |
| `test_app_graceful_shutdown_on_sigterm` | clean shutdown ≤90s grace |
| `test_app_refuses_boot_with_missing_openbao_secrets` | secret-binding fail-fast |
| `test_app_self_observability_emits` | runtime metrics flowing to observability µservice |

## Halt Conditions

- App boots with raw secrets in env — refactor.
- Workers run in same task as REST handler (no isolation) — refactor.

## Next IP

[`IP-012-autonomy-tier-gate.md`](IP-012-autonomy-tier-gate.md)

## References

- `PRD.md` BC layer mapping.
- All upstream IP-003 through IP-010.
