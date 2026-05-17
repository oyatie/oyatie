---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-audit-chain-substrate
impl_plan_id: IP-003-emission-kernel
status: pending
owner: axis-audit-chain
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, data-class]
---

# IP-003: oya-audit-chain-emission-kernel

## Intent

Scaffold the `kernel` layer crate for `emission` BC. Port traits + entities + errors. Zero I/O. Foundation for all downstream `emission`-BC layers.

## Crate Naming

```
NAME: oya-audit-chain-emission-kernel
JUSTIFICATION:
- microservice = audit-chain (microservices/audit-chain/)
- bc-tokens = emission: primary BC per PRD §"Bounded Contexts"
- layer = kernel: ADR-0105 13-value enum
- exemptions claimed: none
```

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/audit-chain/src/crates/oya-audit-chain-emission-kernel/Cargo.toml` | create |
| `.../src/lib.rs` | create — module + `pub use` surface |
| `.../src/entities.rs` | create — `AuditEvent`, `EventEnvelope`, `Period`, `Principal`, `EventClass` with `data_class` annotations |
| `.../src/ports.rs` | create — port traits (sealed): `AuditEmitter`, `WalWriter`, `PrincipalResolver` |
| `.../src/errors.rs` | create — error variants |
| `Cargo.toml` (workspace) | update — add to `[workspace.members]` |
| `microservices/audit-chain/catalog/oya-audit-chain-emission-kernel.yaml` | create |

## Code Shape

```rust
// src/lib.rs
pub mod entities;
pub mod errors;
pub mod ports;

pub use entities::{AuditEvent, EventClass, EventEnvelope, Period, Principal};
pub use errors::{EmissionError, PersistError, ValidationError};
pub use ports::{AuditEmitter, PrincipalResolver, WalWriter};

#[doc(hidden)]
mod sealed { pub trait Sealed {} }
```

```rust
// src/entities.rs (excerpt)
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub tenant_id: TenantId,
    #[data_class(INTERNAL_ONLY)]
    pub source_microservice: String,
    #[data_class(INTERNAL_ONLY)]
    pub event_class: EventClass,
    #[data_class(SENSITIVE_PIPA_ART23)]
    pub principal: Principal,
    #[data_class(INTERNAL_ONLY)]  // payload bytes opaque at this layer
    pub payload: Vec<u8>,
    #[data_class(INTERNAL_ONLY)]
    pub payload_data_class: DataClass,
    #[data_class(INTERNAL_ONLY)]
    pub pack: PackId,
    #[data_class(AUDIT)]
    pub emitted_at: chrono::DateTime<chrono::Utc>,
    #[data_class(PII_QUASI_IDENTIFIER)]
    pub subject_hash: Option<String>,
    #[data_class(INTERNAL_ONLY)]
    pub entity_ref: Option<String>,
    #[data_class(INTERNAL_ONLY)]
    pub idempotency_key: ulid::Ulid,
}

// + EventEnvelope, Period, Principal, EventClass, TenantId, PackId, DataClass
```

```rust
// src/ports.rs
use async_trait::async_trait;
use crate::sealed::Sealed;
use crate::entities::*;
use crate::errors::*;

#[async_trait]
pub trait AuditEmitter: Send + Sync + Sealed {
    async fn emit(&self, event: AuditEvent) -> Result<EmitReceipt, EmissionError>;
}

#[async_trait]
pub trait WalWriter: Send + Sync + Sealed {
    async fn write(&self, envelope: EventEnvelope) -> Result<(), PersistError>;
}

#[async_trait]
pub trait PrincipalResolver: Send + Sync + Sealed {
    async fn resolve_from_spiffe(&self, spiffe_id: &str) -> Result<Principal, ValidationError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-audit-chain-emission-kernel --all-features
cargo build -p oya-audit-chain-emission-kernel --all-features
cargo clippy -p oya-audit-chain-emission-kernel --all-features -- -D warnings
cargo nextest run -p oya-audit-chain-emission-kernel --all-features
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-audit-chain-emission-kernel
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-audit-chain-emission-kernel
```

## Test Plan

Per PHASE-01 kernel class: 1 test per public type + 1 per port trait + sealed-trait smoke. Coverage 90% line / 80% branch.

## References

- Bominal ADR-0028 + ADR-0003.
- ADR-0056 BNF v4.1; ADR-0105 13-layer enum.
- PRD §"Bounded Contexts" port-trait table.
