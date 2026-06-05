---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agentic-slo-gated-promotion
impl_plan_id: IP-009-slo-engine-app
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-observability
acceptance_lanes: [cargo-check, cargo-build, cargo-nextest, lean-a1, composition-root-only]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: oya-observability-slo-engine-app

## Intent

Composition-root binary. Wires worker + rest + adapter clients via dependency injection. Only `[[bin]]` shape per ADR-0105 §"Amendment 2026-05-15".

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/observability/src/crates/oya-observability-slo-engine-app/Cargo.toml` | create — `[[bin]]` only |
| `.../src/main.rs` | create — composition root |
| `.../src/config.rs` | create — env + OpenBao secret loading |
| `microservices/observability/catalog/oya-observability-slo-engine-app.yaml` | create |
| `Cargo.toml` (workspace) | update |

## Crate Naming

```
NAME: oya-observability-slo-engine-app
JUSTIFICATION: microservice=observability; bc=slo-engine; layer=app (composition-root binary per ADR-0105 §"Amendment 2026-05-15")
```

## Code Shape

```rust
// src/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Config::load_from_env_and_openbao().await?;
    let mimir_client = oya_observability_slo_engine_adapter_mimir::MimirClient::new(cfg.mimir.clone());
    let yaml_reader = oya_observability_slo_engine_adapter::OpenSloYamlReader::new(cfg.slos_dir.clone());
    let evaluator = oya_observability_slo_engine_usecase::EvaluateUseCase::new(yaml_reader.clone(), mimir_client.clone());
    let rest = oya_observability_slo_engine_rest::serve(evaluator.clone(), cfg.rest.clone());
    let worker = oya_observability_slo_engine_worker::run(WorkerDeps {
        evaluate_use_case: evaluator,
        // ...
    });
    tokio::try_join!(rest, worker)?;
    Ok(())
}
```

## Acceptance Gates

```bash
cargo build -p oya-observability-slo-engine-app --release
cargo nextest run -p oya-observability-slo-engine-app
buck2 build //:quality-lane-registry-authority-check # lane=composition-root-only --crate oya-observability-slo-engine-app
```

## Test Plan

Per PHASE-01 app class: composition-root smoke + 1 startup-and-shutdown smoke. Coverage 60% line (mostly wiring).

| Test | Verifies |
|---|---|
| `test_compose_and_drop` | smoke; no panics; clean shutdown signal |
| `test_main_with_failing_dependency` | mocked failing Mimir → app exits non-zero, logs root cause |

## Halt Conditions

- Composition-root contains any business logic — refactor to usecase/domain
- Direct adapter cross-imports (e.g., rest imports adapter directly) — refactor through ports

## Next IP

[`IP-010-promotion-eligibility-ledger.md`](IP-010-promotion-eligibility-ledger.md)

## References

- ADR-0105 §"Amendment 2026-05-15" (composition-root → -app suffix)
- PRD §"Bounded Contexts"
