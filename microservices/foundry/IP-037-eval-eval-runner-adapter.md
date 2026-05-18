---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-007-eval-runner-adapter
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: oya-foundry-eval-eval-runner-adapter

## Intent

Protocol-neutral kernel-port implementations: filesystem eval-set reader; provider-route resolver against foundry-providers via Workflow event topology; foundry-evidence client.

## ChangeSet boundary

`microservices/foundry/src/crates/oya-foundry-eval-eval-runner-adapter/`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-eval-eval-runner-adapter/Cargo.toml` | create |
| `src/crates/oya-foundry-eval-eval-runner-adapter/src/lib.rs` | create |
| `src/crates/oya-foundry-eval-eval-runner-adapter/src/eval_set_reader.rs` | create — filesystem reader |
| `src/crates/oya-foundry-eval-eval-runner-adapter/src/route_resolver.rs` | create — foundry-providers client |
| `src/crates/oya-foundry-eval-eval-runner-adapter/src/evidence_emitter.rs` | create — foundry-evidence client |
| `catalog/oya-foundry-eval-eval-runner-adapter.yaml` | create |

## Test Plan

85% line: mocked-I/O round-trip tests + integration tests against fixture filesystem + fake foundry-evidence sink.
