---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-006-eval-runner-api
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: oya-foundry-eval-eval-runner-api

## Intent

Protocol-neutral typed I/O contracts consumed by rest + sdk. Depends on kernel only.

## ChangeSet boundary

`microservices/foundry/src/crates/oya-foundry-eval-eval-runner-api/`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-eval-eval-runner-api/Cargo.toml` | create |
| `src/crates/oya-foundry-eval-eval-runner-api/src/lib.rs` | create |
| `src/crates/oya-foundry-eval-eval-runner-api/src/requests.rs` | create — `PublishGateVerdictRequest`, `TriggerEvalRunRequest`, `EvalRunHistoryRequest` |
| `src/crates/oya-foundry-eval-eval-runner-api/src/responses.rs` | create — `PublishGateVerdict`, `EvalRunReceipt`, `EvalRunHistoryResponse` |
| `src/crates/oya-foundry-eval-eval-runner-api/src/error_variants.rs` | create — `ApiError` enum |
| `catalog/oya-foundry-eval-eval-runner-api.yaml` | create |

## Test Plan

90% line: typed-contract roundtrip; OpenAPI schema-conformance against `contracts/openapi/eval-runner.yaml`.

## Next IPs (parallel-eligible after this IP)

- [`IP-007-eval-runner-adapter.md`](IP-007-eval-runner-adapter.md)
- [`IP-008-eval-runner-adapter-s3.md`](IP-008-eval-runner-adapter-s3.md)
- [`IP-009-eval-runner-adapter-gpu.md`](IP-009-eval-runner-adapter-gpu.md)
