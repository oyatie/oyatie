---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agentic-slo-gated-promotion
impl_plan_id: IP-003-slo-engine-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-observability
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: oya-observability-slo-engine-kernel

## Intent

Scaffold the `kernel` layer crate per ADR-0105: port traits (sealed) + entity types + value objects + error types. Zero I/O. Zero business logic. Foundation that every other slo-engine layer crate depends on.

## ChangeSet boundary

One new Rust crate at `microservices/observability/src/crates/oya-observability-slo-engine-kernel/`. Workspace member added to root `Cargo.toml`. Catalog row at `microservices/observability/catalog/oya-observability-slo-engine-kernel.yaml`. No downstream consumers in this IP; they begin in IP-004+.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/observability/src/crates/oya-observability-slo-engine-kernel/Cargo.toml` | create | `[package]` + minimal deps (`async-trait`, `serde`) |
| `microservices/observability/src/crates/oya-observability-slo-engine-kernel/src/lib.rs` | create | module declarations + `pub use` surface |
| `microservices/observability/src/crates/oya-observability-slo-engine-kernel/src/entities.rs` | create | `SloTarget`, `BurnRateWindow`, `EligibilityVerdict`, `ReleasePointer`, `MimirTenant` with `data_class` annotations |
| `microservices/observability/src/crates/oya-observability-slo-engine-kernel/src/ports.rs` | create | port trait declarations (sealed; all 6 traits per PRD) |
| `microservices/observability/src/crates/oya-observability-slo-engine-kernel/src/errors.rs` | create | error variants per port + entity |
| `Cargo.toml` (workspace) | update | add `microservices/observability/src/crates/oya-observability-slo-engine-kernel` to `[workspace.members]` |
| `microservices/observability/catalog/oya-observability-slo-engine-kernel.yaml` | create | catalog row |

## Crate Naming

```
NAME: oya-observability-slo-engine-kernel
JUSTIFICATION:
- microservice = observability (microservices/observability/)
- bc-tokens = slo-engine (primary BC per PRD §"Bounded Contexts")
- layer = kernel (ADR-0105 13-value enum: inner/pure; port traits + entities only)
- exemptions claimed: none
```

## Code Shape

```rust
// src/lib.rs
pub mod entities;
pub mod errors;
pub mod ports;

pub use entities::{
    BurnRateWindow, EligibilityVerdict, MimirTenant, ReleasePointer, SloTarget,
};
pub use errors::{KernelError, PromqlError, RepositoryError};
pub use ports::{
    BurnRateEvaluator, EligibilityVerdictEmitter, MimirTenantResolver,
    PrometheusClient, ReleasePointerStore, SloTargetRepository,
};

#[doc(hidden)]
mod sealed {
    pub trait Sealed {}
}
```

```rust
// src/entities.rs (excerpt)
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SloTarget {
    #[data_class(INTERNAL_ONLY)]
    pub microservice: String,
    #[data_class(INTERNAL_ONLY)]
    pub sli: SliType,
    #[data_class(INTERNAL_ONLY)]
    pub target: f64,
    #[data_class(INTERNAL_ONLY)]
    pub window: Window,
    #[data_class(INTERNAL_ONLY)]
    pub error_budget: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SliType { Availability, Latency, Correctness, Freshness }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EligibilityVerdict {
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub microservice: String,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub source_sha: String,
    #[data_class(INTERNAL_ONLY)]
    pub target_env: Environment,
    #[data_class(AUDIT)]
    pub verdict: Verdict,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub burn_rate_snapshot: BurnRateSnapshot,
    #[data_class(AUDIT)]
    pub evaluated_at: chrono::DateTime<chrono::Utc>,
}
// ... 4 more entities
```

```rust
// src/ports.rs
use async_trait::async_trait;
use crate::sealed::Sealed;
use crate::entities::*;
use crate::errors::*;

#[async_trait]
pub trait SloTargetRepository: Send + Sync + Sealed {
    async fn load_for_microservice(&self, ms: &str) -> Result<Vec<SloTarget>, RepositoryError>;
}

#[async_trait]
pub trait PrometheusClient: Send + Sync + Sealed {
    async fn instant_query(&self, promql: &str, tenant: &MimirTenant) -> Result<InstantVector, PromqlError>;
    async fn range_query(&self, promql: &str, window: BurnRateWindow, tenant: &MimirTenant) -> Result<RangeVector, PromqlError>;
}

#[async_trait]
pub trait BurnRateEvaluator: Send + Sync + Sealed {
    async fn evaluate(&self, target: &SloTarget, env: Environment, sha: &Sha) -> Result<EligibilityVerdict, KernelError>;
}

#[async_trait]
pub trait EligibilityVerdictEmitter: Send + Sync + Sealed {
    async fn emit(&self, verdict: &EligibilityVerdict) -> Result<(), KernelError>;
}

#[async_trait]
pub trait ReleasePointerStore: Send + Sync + Sealed {
    async fn read(&self, ms: &str, env: Environment) -> Result<ReleasePointer, RepositoryError>;
    async fn advance(&self, ms: &str, env: Environment, sha: &Sha) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait MimirTenantResolver: Send + Sync + Sealed {
    async fn resolve(&self, principal: &Principal) -> Result<MimirTenant, KernelError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-observability-slo-engine-kernel --all-features
cargo build -p oya-observability-slo-engine-kernel --all-features
cargo clippy -p oya-observability-slo-engine-kernel --all-features -- -D warnings
cargo nextest run -p oya-observability-slo-engine-kernel --all-features
cargo deny check
cargo doc -p oya-observability-slo-engine-kernel --no-deps
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-observability-slo-engine-kernel
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-observability-slo-engine-kernel
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-observability-slo-engine-kernel
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-observability-slo-engine-kernel
```

## Test Plan

Per PHASE-01 §"Per-IP Test Coverage Threshold" kernel class: 1 test per public type + 1 per port trait + 1 sealed-trait smoke. Coverage 90% line / 80% branch.

| Test | Verifies |
|---|---|
| `test_slo_target_construction` | entity invariants |
| `test_eligibility_verdict_serde` | serde roundtrip |
| `test_burn_rate_window_arithmetic_pure` | no I/O |
| `test_port_traits_sealed` | external crates cannot impl sealed traits |
| `test_data_class_annotations_present` | every public field has `#[data_class(..)]` |

## Halt Conditions

- BNF v4.1 naming violation — refer to feedback_naming_justification.md
- Any port trait introduces business logic — refactor to domain/usecase
- Any I/O reachable from kernel — refactor

## Next IP

[`IP-004-slo-engine-domain.md`](IP-004-slo-engine-domain.md)

## References

- ADR-0056 BNF v4.1; ADR-0105 13-layer enum; ADR-0130 §"Naming justification"
- PRD §"Bounded Contexts" port-trait table
- Bominal ADR-0028 (data-class taxonomy)
