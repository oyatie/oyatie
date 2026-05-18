---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-012-eval-runner-sdk
status: pending
execution_unit: ChangeSet
owner: axis-foundry + axis-developer-experience
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: oya-foundry-eval-eval-runner-sdk

## Intent

Rust client SDK for capability owners + tenant operators; closes the OpenAI-Evals / Anthropic-evals / LangSmith / Patronus / Braintrust SDK gap. TS + Python bindings via wasm-bindgen + PyO3 (M02).

## ChangeSet boundary

`microservices/foundry/src/crates/oya-foundry-eval-eval-runner-sdk/`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-eval-eval-runner-sdk/Cargo.toml` | create — `reqwest`, `serde`, `oya-foundry-eval-eval-runner-api` |
| `src/crates/oya-foundry-eval-eval-runner-sdk/src/lib.rs` | create |
| `src/crates/oya-foundry-eval-eval-runner-sdk/src/client.rs` | create — Client + builder |
| `src/crates/oya-foundry-eval-eval-runner-sdk/src/operations.rs` | create — per-endpoint methods |
| `src/crates/oya-foundry-eval-eval-runner-sdk/src/error.rs` | create — typed errors + retry helpers |
| `src/crates/oya-foundry-eval-eval-runner-sdk/src/observability.rs` | create — OTel spans |
| `catalog/oya-foundry-eval-eval-runner-sdk.yaml` | create |

## Test Plan

90% line: client-side roundtrip against `oya-foundry-eval-eval-runner-rest` (workspace integration test).
