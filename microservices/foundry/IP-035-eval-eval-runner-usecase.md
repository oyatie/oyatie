---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-005-eval-runner-usecase
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, port-location, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: oya-foundry-eval-eval-runner-usecase

## Intent

Orchestrators reading eval-sets via registry port; dispatching cases via case-dispatcher port; composing aggregates via domain; emitting via evidence port. Depends on kernel + domain.

## ChangeSet boundary

`microservices/foundry/src/crates/oya-foundry-eval-eval-runner-usecase/`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-eval-eval-runner-usecase/Cargo.toml` | create |
| `src/crates/oya-foundry-eval-eval-runner-usecase/src/lib.rs` | create |
| `src/crates/oya-foundry-eval-eval-runner-usecase/src/orchestrator.rs` | create — execute_eval_run orchestrator |
| `src/crates/oya-foundry-eval-eval-runner-usecase/src/publish_gate.rs` | create — verdict computation logic |
| `src/crates/oya-foundry-eval-eval-runner-usecase/src/nightly_scheduler.rs` | create — cadence orchestrator |
| `src/crates/oya-foundry-eval-eval-runner-usecase/src/eu_ai_act_emitter.rs` | create — §15 + §17 emission |
| `catalog/oya-foundry-eval-eval-runner-usecase.yaml` | create |

## Test Plan

90% line / 80% branch: mocked-port scenario tests covering happy path + each error variant + EU AI Act §15/§17 emission shape.

## Acceptance Gates

Same as IP-003 + usecase-layer-specific lane validations.

## Next IP

[`IP-006-eval-runner-api.md`](IP-006-eval-runner-api.md)
