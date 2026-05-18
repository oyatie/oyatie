---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-004-eval-runner-domain
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, port-location, layer-correctness]
---

# IP-004: oya-foundry-eval-eval-runner-domain

## Intent

Pure domain layer: aggregate computation (pass-rate, per-cohort rollup, threshold check). Depends on kernel only. Zero I/O. Zero async.

## ChangeSet boundary

`microservices/foundry/src/crates/oya-foundry-eval-eval-runner-domain/`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-eval-eval-runner-domain/Cargo.toml` | create |
| `src/crates/oya-foundry-eval-eval-runner-domain/src/lib.rs` | create |
| `src/crates/oya-foundry-eval-eval-runner-domain/src/aggregate.rs` | create — pass-rate math |
| `src/crates/oya-foundry-eval-eval-runner-domain/src/cohort.rs` | create — per-cohort rollup |
| `src/crates/oya-foundry-eval-eval-runner-domain/src/threshold.rs` | create — pass_threshold check |
| `catalog/oya-foundry-eval-eval-runner-domain.yaml` | create |

## Test Plan

95% line / 90% branch: pure-function regression tests on aggregate + threshold arithmetic.

## Acceptance Gates

Same as IP-003 + `cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-foundry-eval-eval-runner-domain` (must depend only on kernel).

## Next IP

[`IP-005-eval-runner-usecase.md`](IP-005-eval-runner-usecase.md)
