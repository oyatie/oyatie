---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-003-capability-executor-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-runtime
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: oya-foundry-runtime-capability-executor-kernel

## Intent

Scaffold the kernel layer crate per ADR-0105: port traits (sealed) + entity types + value objects + error types. Zero I/O. Zero business logic. Foundation for every other executor-BC layer crate.

## ChangeSet boundary

One new Rust crate at `microservices/intelligence/src/crates/oya-foundry-runtime-capability-executor-kernel/`. Workspace member added to root Cargo.toml. Catalog row.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-runtime-capability-executor-kernel/Cargo.toml` | create |
| `.../src/lib.rs` | create (module declarations + pub use surface) |
| `.../src/entities.rs` | create (Capability, Invocation, InvocationStep, InvocationResult, AutonomyLevel) |
| `.../src/ports.rs` | create (CapabilityResolver, ProviderInvoker, GuardrailChecker, EvidenceEmitter, AutonomyGate) |
| `.../src/errors.rs` | create |
| `Cargo.toml` (workspace) | update |
| `catalog/oya-foundry-runtime-capability-executor-kernel.yaml` | create |

## Crate Naming

```
NAME: oya-foundry-runtime-capability-executor-kernel
JUSTIFICATION:
- microservice = foundry-runtime
- bc-tokens = capability-executor (primary BC)
- layer = kernel (ADR-0105: inner; port traits + entities only)
- exemptions claimed: none
```

## Code Shape

```rust
// src/lib.rs
pub mod entities;
pub mod errors;
pub mod ports;

pub use entities::{
    AutonomyLevel, Capability, EuAiActClass, Invocation, InvocationStep,
    InvocationResult, Sha,
};
pub use errors::{KernelError, ProviderError, GuardrailError, AutonomyError};
pub use ports::{
    AutonomyGate, CapabilityResolver, EvidenceEmitter, GuardrailChecker,
    ProviderInvoker,
};

#[doc(hidden)]
mod sealed { pub trait Sealed {} }
```

```rust
// src/entities.rs (excerpt)
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Capability {
    #[data_class(INTERNAL_ONLY)]
    pub capability_id: String,
    #[data_class(SENSITIVE_PIPA_ART23)]
    pub tenant_id: String,
    #[data_class(INTERNAL_ONLY)]
    pub declared_autonomy_level: AutonomyLevel,
    #[data_class(INTERNAL_ONLY)]
    pub version: String,
    #[data_class(INTERNAL_ONLY)]
    pub eu_ai_act_class: EuAiActClass,
    #[data_class(AUDIT)]
    pub signature: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AutonomyLevel { T0 = 0, T1 = 1, T2 = 2, T3 = 3, T4 = 4 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Invocation {
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub invocation_id: String,
    #[data_class(SENSITIVE_PIPA_ART23)]
    pub tenant_id: String,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub capability_id: String,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub session_id: Option<String>,
    #[data_class(AUDIT)]
    pub autonomy_level_used: AutonomyLevel,
    #[data_class(AUDIT)]
    pub started_at: chrono::DateTime<chrono::Utc>,
}
// ... InvocationStep, InvocationResult, EuAiActClass
```

```rust
// src/ports.rs
use async_trait::async_trait;
use crate::sealed::Sealed;
use crate::entities::*;
use crate::errors::*;

#[async_trait]
pub trait CapabilityResolver: Send + Sync + Sealed {
    async fn resolve(&self, tenant_id: &str, capability_id: &str) -> Result<Capability, KernelError>;
}

#[async_trait]
pub trait ProviderInvoker: Send + Sync + Sealed {
    async fn invoke(&self, capability: &Capability, input: &serde_json::Value) -> Result<serde_json::Value, ProviderError>;
}

#[async_trait]
pub trait GuardrailChecker: Send + Sync + Sealed {
    async fn check(&self, payload: &serde_json::Value, direction: GuardrailDirection) -> Result<GuardrailVerdict, GuardrailError>;
}

#[async_trait]
pub trait EvidenceEmitter: Send + Sync + Sealed {
    async fn emit(&self, event: &InvocationStep) -> Result<(), KernelError>;
}

#[async_trait]
pub trait AutonomyGate: Send + Sync + Sealed {
    async fn check(&self, tenant_id: &str, requested: AutonomyLevel) -> Result<AutonomyDecision, AutonomyError>;
}

pub enum GuardrailDirection { PreFlight, PostFlight }
pub enum GuardrailVerdict { Permit, Block { reason: String } }
pub enum AutonomyDecision { Permit { ceiling: AutonomyLevel }, Refuse { ceiling: AutonomyLevel } }
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-runtime-capability-executor-kernel --all-features
cargo build -p oya-foundry-runtime-capability-executor-kernel --all-features
cargo clippy -p oya-foundry-runtime-capability-executor-kernel --all-features -- -D warnings
cargo nextest run -p oya-foundry-runtime-capability-executor-kernel --all-features
cargo deny check
cargo doc -p oya-foundry-runtime-capability-executor-kernel --no-deps
buck2 build //:quality-lane-registry-authority-check # lane=lean-a1 --crate oya-foundry-runtime-capability-executor-kernel
buck2 build //:quality-lane-registry-authority-check # lane=port-location --crate oya-foundry-runtime-capability-executor-kernel
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --crate oya-foundry-runtime-capability-executor-kernel
buck2 build //:quality-lane-registry-authority-check # lane=data-class --crate oya-foundry-runtime-capability-executor-kernel
```

## Test Plan

Per PHASE-01 kernel class: 1 test per public type + 1 per port trait + 1 sealed-trait smoke. 90% line / 80% branch.

| Test | Verifies |
|---|---|
| `test_capability_construction` | entity invariants |
| `test_autonomy_level_ordering` | T0 < T1 < T2 < T3 < T4 |
| `test_invocation_serde` | serde roundtrip |
| `test_port_traits_sealed` | external crates cannot impl sealed traits |
| `test_data_class_annotations_present` | every public field has `#[data_class(..)]` |

## Halt Conditions

- BNF v4.1 naming violation.
- Any port trait introduces business logic — refactor to domain/usecase.
- Any I/O reachable from kernel — refactor.

## Next IP

[`IP-004-capability-executor-domain-and-usecase.md`](IP-004-capability-executor-domain-and-usecase.md)

## References

- ADR-0022 (autonomy tiers); ADR-0056 (BNF v4.1); ADR-0105 (13-layer); ADR-0131.
- PRD §"Bounded Contexts" port-trait table.
- Bominal ADR-0028 (data-class taxonomy).

## Wave 15 counterpart anchor

- Counterparts: OpenAI Assistants, AWS Bedrock Agents, and Cloudflare Workers sandboxing.
- Gap closure: this IP closes session/run execution, capability isolation, and sandbox accounting with Oyatie tenant, Cedar, and evidence-chain controls.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
