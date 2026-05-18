---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-010-eval-runner-rest
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, openapi-conformance, cedar-policy-coverage]
---

# IP-010: oya-foundry-eval-eval-runner-rest

## Intent

HTTP handler/route layer; consumes -api types; serves the OpenAPI surface at `contracts/openapi/eval-runner.yaml`. Cedar policy enforced on every endpoint.

## ChangeSet boundary

`microservices/foundry/src/crates/oya-foundry-eval-eval-runner-rest/`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-eval-eval-runner-rest/Cargo.toml` | create — `axum`, `tower`, `cedar-policy` |
| `src/crates/oya-foundry-eval-eval-runner-rest/src/lib.rs` | create |
| `src/crates/oya-foundry-eval-eval-runner-rest/src/handlers/publish_gate.rs` | create |
| `src/crates/oya-foundry-eval-eval-runner-rest/src/handlers/eval_runs.rs` | create |
| `src/crates/oya-foundry-eval-eval-runner-rest/src/handlers/eu_ai_act.rs` | create |
| `src/crates/oya-foundry-eval-eval-runner-rest/src/policy_middleware.rs` | create — Cedar enforcement |
| `catalog/oya-foundry-eval-eval-runner-rest.yaml` | create |

## Test Plan

90% line: handler-route happy + error paths; Cedar policy enforcement edge cases (cross-tenant attempt → 403; valid → 200); OpenAPI schema-conformance test.
