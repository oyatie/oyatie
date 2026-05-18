---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-004-lane-runtime-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, port-location, layer-correctness, data-class]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: oya-governance-lane-runtime-{kernel,domain}

## Intent

Fill in the kernel + domain layers of the `lane-runtime` BC. Kernel = port traits + entity types (zero I/O). Domain = pure scheduling math, retry-budget arithmetic, matrix-fanout calculator.

## ChangeSet boundary

2 crates: `oya-governance-lane-runtime-kernel` + `oya-governance-lane-runtime-domain`. Code-only; no workspace member change (scaffolding already done in IP-001).

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/governance/src/crates/oya-governance-lane-runtime-kernel/src/entities.rs` | create | `LaneId`, `LaneRun`, `LaneRequest`, `LaneVerdict`, `RunnerProfile` with `data_class` annotations |
| `microservices/governance/src/crates/oya-governance-lane-runtime-kernel/src/ports.rs` | create | port trait declarations (sealed): `LaneRegistry`, `LaneDispatcher`, `RunnerProfileStore` |
| `microservices/governance/src/crates/oya-governance-lane-runtime-kernel/src/errors.rs` | create | error variants |
| `microservices/governance/src/crates/oya-governance-lane-runtime-domain/src/scheduling.rs` | create | matrix-fanout math; retry-budget arithmetic (per `policy/lane-execution.md` Invariant 3 + 6) |
| `microservices/governance/src/crates/oya-governance-lane-runtime-domain/src/queueing.rs` | create | queue-depth + per-µservice fairness algorithm |

## Code Shape

```rust
// oya-governance-lane-runtime-kernel/src/entities.rs (excerpt)
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LaneId(String);

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity { Blocker, Warn, Info }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Verdict { Pass, Fail, Error, Skipped, Timeout }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LaneRun {
    #[data_class(AUDIT)]      pub run_id: uuid::Uuid,
    #[data_class(INTERNAL_ONLY)] pub lane_id: LaneId,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)] pub microservice: String,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)] pub sha: String,
    #[data_class(AUDIT)]      pub verdict: Verdict,
    #[data_class(INTERNAL_ONLY)] pub duration_ms: u64,
    #[data_class(INTERNAL_ONLY)] pub runner_profile: RunnerProfile,
    #[data_class(INTERNAL_ONLY)] pub runner_id: String,
    #[data_class(AUDIT)]      pub started_at: chrono::DateTime<chrono::Utc>,
}
// ... 3 more entities
```

```rust
// oya-governance-lane-runtime-kernel/src/ports.rs
use async_trait::async_trait;
use crate::sealed::Sealed;
use crate::entities::*;
use crate::errors::*;

#[async_trait]
pub trait LaneRegistry: Send + Sync + Sealed {
    async fn list(&self) -> Result<Vec<LaneSummary>, KernelError>;
    async fn get(&self, id: &LaneId) -> Result<Option<LaneSummary>, KernelError>;
    async fn register(&self, summary: LaneSummary) -> Result<(), KernelError>;
}

#[async_trait]
pub trait LaneDispatcher: Send + Sync + Sealed {
    async fn dispatch(&self, req: LaneRequest) -> Result<LaneRun, KernelError>;
    async fn cancel(&self, run_id: &uuid::Uuid) -> Result<(), KernelError>;
}

#[async_trait]
pub trait RunnerProfileStore: Send + Sync + Sealed {
    async fn get(&self, profile: &RunnerProfile) -> Result<RunnerProfileSpec, KernelError>;
}
```

```rust
// oya-governance-lane-runtime-domain/src/scheduling.rs
use oya_governance_lane_runtime_kernel::*;

pub fn compute_matrix_fanout(req: &PullRequestContext, lanes: &[LaneSummary]) -> Vec<LaneRequest> {
    // pure function; one entry per (lane, microservice) tuple within PR scope
    // bounded by max_replicas cap (200) per capacity-model.md
    todo!()
}

pub fn retry_budget(profile: &RunnerProfile, attempt: u32) -> Option<chrono::Duration> {
    // exponential-backoff with cap at lane's 60s wall-clock bound
    todo!()
}
```

## Acceptance Gates

```bash
cargo check -p oya-governance-lane-runtime-kernel --all-features
cargo check -p oya-governance-lane-runtime-domain --all-features
cargo build -p oya-governance-lane-runtime-kernel --all-features
cargo build -p oya-governance-lane-runtime-domain --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run -p oya-governance-lane-runtime-kernel
cargo nextest run -p oya-governance-lane-runtime-domain
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-governance-lane-runtime-kernel
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-governance-lane-runtime-kernel
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-governance-lane-runtime-domain
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-governance-lane-runtime-kernel
```

## Test Plan

Per PHASE-01 §"Per-IP Test Coverage Threshold" kernel + domain class: 90% line + 80% branch coverage minimum.

| Test | Verifies |
|---|---|
| `test_lane_run_serde_roundtrip` | entity serde stability |
| `test_data_class_annotations_present` | every public field annotated |
| `test_port_traits_sealed` | external crates cannot impl sealed traits |
| `test_matrix_fanout_bounded_at_200` | capacity-model bound |
| `test_retry_budget_capped_at_60s` | Invariant 3 bound |
| `test_per_microservice_fairness_30pct_max` | T-D-01 mitigation |

## Halt Conditions

- Port trait introduces I/O — refactor to adapter.
- Domain layer imports I/O dep — refactor.

## Next IP

[`IP-005-lane-runtime-usecase-adapter-rest.md`](IP-005-lane-runtime-usecase-adapter-rest.md)

## References

- `microservices/governance/PRD.md` §"Bounded Contexts" lane-runtime.
- `microservices/governance/policy/lane-execution.md`.
- ADR-0105 13-layer enum.
- Bominal ADR-0028 (data-class taxonomy).
