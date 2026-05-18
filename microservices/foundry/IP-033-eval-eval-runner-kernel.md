---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-003-eval-runner-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: oya-foundry-eval-eval-runner-kernel

## Intent

Scaffold `kernel` crate per ADR-0105: port traits (sealed) + entity types + value objects + errors. Zero I/O. Zero business logic.

## ChangeSet boundary

One new Rust crate at `microservices/foundry/src/crates/oya-foundry-eval-eval-runner-kernel/`.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `src/crates/oya-foundry-eval-eval-runner-kernel/Cargo.toml` | create | `[package]` + `async-trait`, `serde`, `chrono`, `uuid` |
| `src/crates/oya-foundry-eval-eval-runner-kernel/src/lib.rs` | create | module declarations + `pub use` |
| `src/crates/oya-foundry-eval-eval-runner-kernel/src/entities.rs` | create | `EvalRun`, `EvalCaseResult`, `EvalAggregate`, `ProviderRoute`, `CohortAggregate` with `data_class` annotations |
| `src/crates/oya-foundry-eval-eval-runner-kernel/src/ports.rs` | create | port trait declarations (sealed; EvalRunner, CaseDispatcher, EvalRunStore, EvalEvidenceEmitter) |
| `src/crates/oya-foundry-eval-eval-runner-kernel/src/errors.rs` | create | error variants |
| `Cargo.toml` (workspace) | update | add to `[workspace.members]` |
| `catalog/oya-foundry-eval-eval-runner-kernel.yaml` | create | catalog row |

## Crate Naming

```
NAME: oya-foundry-eval-eval-runner-kernel
JUSTIFICATION:
- microservice = foundry-eval (microservices/foundry/)
- bc-tokens = eval-runner (primary BC per PRD §"Bounded Contexts")
- layer = kernel (ADR-0105 13-value enum)
- exemptions claimed: none
```

## Code Shape

```rust
// src/lib.rs
pub mod entities;
pub mod errors;
pub mod ports;

pub use entities::{
    CohortAggregate, EvalAggregate, EvalCaseResult, EvalRun, ProviderRoute, Verdict,
};
pub use errors::{KernelError, DispatchError, StoreError};
pub use ports::{CaseDispatcher, EvalEvidenceEmitter, EvalRunStore, EvalRunner};

#[doc(hidden)]
mod sealed {
    pub trait Sealed {}
}
```

```rust
// src/entities.rs (excerpt)
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvalRun {
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub run_id: uuid::Uuid,
    #[data_class(INTERNAL_ONLY)]
    pub capability_id: String,
    #[data_class(INTERNAL_ONLY)]
    pub version: String,
    #[data_class(INTERNAL_ONLY)]
    pub route: ProviderRoute,
    #[data_class(AUDIT)]
    pub started_at: chrono::DateTime<chrono::Utc>,
    #[data_class(AUDIT)]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub aggregate: Option<EvalAggregate>,
    #[data_class(AUDIT)]
    pub passed: bool,
    #[data_class(AUDIT)]
    pub eu_ai_act_section_15_accuracy_metric: Option<f64>,
    #[data_class(AUDIT)]
    pub eu_ai_act_section_17_logging_payload_ref: Option<String>,
    #[data_class(AUDIT)]
    pub signature: Vec<u8>,
}
```

```rust
// src/ports.rs
use async_trait::async_trait;
use crate::sealed::Sealed;
use crate::entities::*;
use crate::errors::*;

#[async_trait]
pub trait EvalRunner: Send + Sync + Sealed {
    async fn execute(&self, capability_id: &str, version: &str, route: ProviderRoute, cohorts: &[String]) -> Result<EvalRun, KernelError>;
}

#[async_trait]
pub trait CaseDispatcher: Send + Sync + Sealed {
    async fn dispatch(&self, case_input: serde_json::Value, route: &ProviderRoute) -> Result<EvalCaseResult, DispatchError>;
}

#[async_trait]
pub trait EvalRunStore: Send + Sync + Sealed {
    async fn put(&self, run: &EvalRun) -> Result<(), StoreError>;
    async fn get(&self, run_id: uuid::Uuid) -> Result<Option<EvalRun>, StoreError>;
    async fn list(&self, capability_id: &str, limit: usize) -> Result<Vec<EvalRun>, StoreError>;
}

#[async_trait]
pub trait EvalEvidenceEmitter: Send + Sync + Sealed {
    async fn emit_run_started(&self, run: &EvalRun) -> Result<(), KernelError>;
    async fn emit_run_completed(&self, run: &EvalRun) -> Result<(), KernelError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-eval-eval-runner-kernel --all-features
cargo build -p oya-foundry-eval-eval-runner-kernel --all-features
cargo clippy -p oya-foundry-eval-eval-runner-kernel --all-features -- -D warnings
cargo nextest run -p oya-foundry-eval-eval-runner-kernel --all-features
cargo deny check
cargo doc -p oya-foundry-eval-eval-runner-kernel --no-deps
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-foundry-eval-eval-runner-kernel
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-foundry-eval-eval-runner-kernel
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-foundry-eval-eval-runner-kernel
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-foundry-eval-eval-runner-kernel
```

## Test Plan

Per PHASE-01 kernel class threshold (90% line / 80% branch):

| Test | Verifies |
|---|---|
| `test_eval_run_construction` | entity invariants |
| `test_eval_run_serde` | serde roundtrip |
| `test_eval_aggregate_serde` | serde roundtrip |
| `test_port_traits_sealed` | external crates cannot impl |
| `test_data_class_annotations_present` | every public field has `#[data_class(..)]` |

## Halt Conditions

- BNF v4.1 naming violation.
- Any port trait introduces business logic.
- Any I/O reachable from kernel.

## Next IP

[`IP-004-eval-runner-domain.md`](IP-004-eval-runner-domain.md)

## References

- ADR-0024, ADR-0056, ADR-0105, ADR-0131.
- PRD §"Bounded Contexts" port-trait table.
