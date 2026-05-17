---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-013-eval-runner-app
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, composition-root-only]
---

# IP-013: oya-foundry-eval-eval-runner-app

## Intent

Composition-root binary that wires worker + rest + adapter clients via clean-arch composition pattern. Per ADR-0105 + clean-architecture-requirements: ONLY place where dependency-graph composition happens.

## ChangeSet boundary

`microservices/foundry-eval/src/crates/oya-foundry-eval-eval-runner-app/`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-eval-eval-runner-app/Cargo.toml` | create — depends on all eval-runner sub-crates |
| `src/crates/oya-foundry-eval-eval-runner-app/src/main.rs` | create — entry-point + signal handling |
| `src/crates/oya-foundry-eval-eval-runner-app/src/wiring.rs` | create — composition root |
| `src/crates/oya-foundry-eval-eval-runner-app/src/config.rs` | create — env + config parsing |
| `catalog/oya-foundry-eval-eval-runner-app.yaml` | create |

## Test Plan

80% line: composition-root smoke test (binary starts + receives a request + emits a verdict).
