---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
impl_plan_id: IP-012-app-composition-roots
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-iac
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: cloud-iac-iac-*-app crates (all 5 BCs)

## Intent

Implement `-app` composition-root binaries for all 5 BCs. Each app wires its BC's worker + rest + adapters. Composition root only — no business logic.

## ChangeSet boundary

Five new crates per ADR-0105: one `-app` per BC. Catalog rows. Dockerfile per app.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-iac/src/crates/cloud-iac-iac-renderer-app/{Cargo.toml,src/main.rs,src/wiring.rs,Dockerfile}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-validator-app/{Cargo.toml,src/main.rs,src/wiring.rs,Dockerfile}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-applier-app/{Cargo.toml,src/main.rs,src/wiring.rs,Dockerfile}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-rollback-app/{Cargo.toml,src/main.rs,src/wiring.rs,Dockerfile}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-registry-app/{Cargo.toml,src/main.rs,src/wiring.rs,Dockerfile}` | create |
| `microservices/cloud-iac/catalog/cloud-iac-iac-*-app.yaml` | create (5 rows) |

## Code Shape

```rust
// applier-app/src/wiring.rs
pub fn assemble() -> Result<ApplierApp, anyhow::Error> {
    let postgres = adapter_postgres::PostgresClient::from_env()?;
    let k8s = adapter::K8sClient::from_kubeconfig()?;
    let argocd = adapter_argocd::ArgoCdClient::from_env()?;
    let event_bus = adapter::EventBusClient::from_env()?;

    let applier = usecase::ApplyOrchestrator::new(k8s, argocd, slsa_verifier, event_bus);
    let rest = rest::ApplierRest::new(applier.clone());
    let worker = worker::ApplierWorker::new(applier);
    Ok(ApplierApp { rest, worker })
}

// main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let app = wiring::assemble()?;
    tokio::try_join!(app.serve_rest(), app.run_worker())?;
    Ok(())
}
```

## Acceptance Gates

```bash
cargo build --release --workspace -p cloud-iac-iac-*-app --all-features
docker build -t cloud-iac-applier:test -f microservices/cloud-iac/src/crates/cloud-iac-iac-applier-app/Dockerfile .
cloud-ci/ci governance gate `layer-correctness` for --microservice cloud-iac is green in the branch-protected `presubmit` context
```

## Test Plan

Per PHASE-01 app class: composition-root smoke + startup-and-shutdown smoke. Coverage 60% (mostly wiring).

| Test | Verifies |
|---|---|
| `test_renderer_app_smoke` | binary starts; serves /health |
| `test_validator_app_smoke` | binary starts; drift loop ticks |
| `test_applier_app_smoke` | binary starts; consumes eligibility events |
| `test_rollback_app_smoke` | binary starts; consumes rollback events |
| `test_registry_app_smoke` | binary starts; serves /health + queries Postgres |

## Halt Conditions

- App imports business logic directly — must invoke usecase via wiring.
- Hardcoded secrets in main.rs — refactor to OpenBao SecretReference.

## Next IP

[`IP-013-sdk-and-observability-slo.md`](IP-013-sdk-and-observability-slo.md)

## References

- ADR-0105 §"app layer".
- PRD §"Bounded Contexts".
