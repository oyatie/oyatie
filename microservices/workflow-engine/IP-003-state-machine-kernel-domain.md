---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-003-state-machine-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness]
---

# IP-003: oya-workflow-engine-state-machine-{kernel,domain,usecase,api,adapter,adapter-postgres}

## Intent

Scaffold the full state-machine BC: kernel (port traits + entities) + domain (pure transition evaluation + invariant checks) + usecase (orchestrators) + api (typed contracts) + adapter + adapter-postgres (checkpoint persistence). State-machine concerns are PURE — no I/O at evaluation layer.

## ChangeSet boundary

6 new Rust crates. Workspace members added. Catalog rows for each.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `src/crates/oya-workflow-engine-state-machine-kernel/{Cargo.toml,src/{lib,entities,ports,errors}.rs}` | create | `Transition`, `TransitionRule`, `StateCheckpoint` entities + `TransitionEngine`, `InvariantValidator`, `StateCheckpointStore` port traits |
| `src/crates/oya-workflow-engine-state-machine-domain/{Cargo.toml,src/{lib,transition_eval,invariant_check}.rs}` | create | pure transition eval over spec + current state |
| `src/crates/oya-workflow-engine-state-machine-usecase/{Cargo.toml,src/{lib,compose}.rs}` | create | orchestrate transition + invariant + checkpoint write via ports |
| `src/crates/oya-workflow-engine-state-machine-api/{Cargo.toml,src/{lib,types,errors}.rs}` | create | typed I/O |
| `src/crates/oya-workflow-engine-state-machine-adapter/{Cargo.toml,src/lib.rs}` | create | protocol-neutral impls |
| `src/crates/oya-workflow-engine-state-machine-adapter-postgres/{Cargo.toml,src/lib.rs}` | create | Postgres checkpoint persistence |
| `microservices/workflow-engine/catalog/oya-workflow-engine-state-machine-{kernel,domain,usecase,api,adapter,adapter-postgres}.yaml` | create | 6 catalog rows |
| `Cargo.toml` (workspace) | update | register 6 crates |

## Acceptance Gates

```bash
cargo check -p oya-workflow-engine-state-machine-kernel ... (×6)
cargo nextest run -p oya-workflow-engine-state-machine-domain --all-features
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-workflow-engine-state-machine-kernel
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-workflow-engine-state-machine-domain
```

## Test Plan

- kernel: 90%/80% coverage
- domain: 95%/90% + property tests on transition determinism
- usecase: 90%/80% with mocked ports
- adapter-postgres: 85%/75% against testcontainer

| Test | Verifies |
|---|---|
| `test_transition_eval_deterministic` | same (state, event, spec) → same next-state |
| `test_invariant_four_eyes` | four-eyes constraint refuses single-approver path |
| `test_checkpoint_persistence_roundtrip` | persist → load returns identical checkpoint |

## Next IP

[`IP-004-execution-engine-kernel-domain.md`](IP-004-execution-engine-kernel-domain.md)

## References

- PRD §"Bounded Contexts" state-machine row
- ADR-0035 (Bominal): Workflow engine
- ADR-0103 (Bominal): Workflow hexagonal migration
