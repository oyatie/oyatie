---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-001-router-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: oya-foundry-providers-router-kernel

## Intent

Scaffold the kernel layer per ADR-0105: port traits (sealed), entities, value objects, error types. Zero I/O. Zero business logic. Foundation for every other router layer crate plus all adapter crates.

## ChangeSet boundary

One new Rust crate at `microservices/foundry/src/crates/oya-foundry-providers-router-kernel/`. Workspace member added to root `Cargo.toml`. Catalog row at `microservices/foundry/catalog/oya-foundry-providers-router-kernel.yaml`. No downstream consumers in this IP; they begin in IP-002+.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/foundry/src/crates/oya-foundry-providers-router-kernel/Cargo.toml` | create |
| `.../src/lib.rs` | create — module surface |
| `.../src/entities.rs` | create — `RoutingRequest`, `RouterDecision`, `ProviderCandidate`, `CapabilityProfile`, `ResidencyConstraint`, `ProviderHealthSnapshot`, `SecretReference`, `Vendor`, `Transport` |
| `.../src/ports.rs` | create — `ProviderInvoker`, `ProviderRouter`, `CredentialResolver`, `HealthMonitor`, `ProviderConfigRepository`, `TokenBucket` |
| `.../src/errors.rs` | create — kernel-side error variants |
| `Cargo.toml` (workspace) | update — register crate |
| `microservices/foundry/catalog/oya-foundry-providers-router-kernel.yaml` | create |

## Crate Naming

```
NAME: oya-foundry-providers-router-kernel
JUSTIFICATION:
- microservice = foundry-providers
- bc-tokens = router (primary BC)
- layer = kernel (ADR-0105 13-value enum; inner/pure; ports + entities only)
- exemptions claimed: none
```

## Code Shape (excerpt)

```rust
// src/lib.rs
pub mod entities;
pub mod errors;
pub mod ports;

pub use entities::{
    CapabilityProfile, ProviderCandidate, ProviderHealthSnapshot,
    ResidencyConstraint, RouterDecision, RoutingRequest, SecretReference,
    Transport, Vendor,
};
pub use errors::{KernelError, ResolverError, RouterError};
pub use ports::{
    CredentialResolver, HealthMonitor, ProviderConfigRepository,
    ProviderInvoker, ProviderRouter, TokenBucket,
};

#[doc(hidden)]
mod sealed { pub trait Sealed {} }
```

```rust
// src/entities.rs (excerpt)
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Vendor { Anthropic, OpenAI, Gemini, InHouse }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Transport { Api, Subscription, InHouse }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoutingRequest {
    pub tenant_id: String,
    pub pack: String,
    pub capability_profile: CapabilityProfile,
    pub request_fingerprint: String, // BLAKE3 hex; no prompt bytes
    pub constraints: RoutingConstraints,
}

// IMPORTANT: SecretReference is a URI only; never carries credential bytes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecretReference { pub uri: String }

// ResolvedCredential is NOT in this crate — it lives in the adapter-openbao
// crate and is deliberately opaque (no Serialize / Display / loggable Debug).
```

```rust
// src/ports.rs
use async_trait::async_trait;
use crate::sealed::Sealed;
use crate::entities::*;
use crate::errors::*;

#[async_trait]
pub trait ProviderRouter: Send + Sync + Sealed {
    async fn decide(&self, req: &RoutingRequest) -> Result<RouterDecision, RouterError>;
}

#[async_trait]
pub trait ProviderInvoker: Send + Sync + Sealed {
    type Request: Send + Sync;
    type Response: Send + Sync;
    async fn invoke(&self, req: Self::Request) -> Result<Self::Response, RouterError>;
}

#[async_trait]
pub trait CredentialResolver: Send + Sync + Sealed {
    type Credential;
    async fn resolve(&self, secret_ref: &SecretReference, caller_ctx: &CallerCtx)
        -> Result<Self::Credential, ResolverError>;
}

#[async_trait]
pub trait HealthMonitor: Send + Sync + Sealed {
    async fn snapshot(&self, vendor: Vendor, transport: Transport, region: &str)
        -> Result<ProviderHealthSnapshot, KernelError>;
}

#[async_trait]
pub trait ProviderConfigRepository: Send + Sync + Sealed {
    async fn load(&self, tenant_id: &str, pack: &str) -> Result<TenantProviderConfig, KernelError>;
}

#[async_trait]
pub trait TokenBucket: Send + Sync + Sealed {
    async fn take(&self, tenant_id: &str, vendor: Vendor) -> Result<bool, KernelError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-providers-router-kernel --all-features
cargo build -p oya-foundry-providers-router-kernel --all-features
cargo clippy -p oya-foundry-providers-router-kernel --all-features -- -D warnings
cargo nextest run -p oya-foundry-providers-router-kernel --all-features
cargo deny check
cargo doc -p oya-foundry-providers-router-kernel --no-deps
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-foundry-providers-router-kernel
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-foundry-providers-router-kernel
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-foundry-providers-router-kernel
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-foundry-providers-router-kernel
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_routing_request_construction` | entity invariants |
| `test_router_decision_serde` | serde roundtrip |
| `test_secret_reference_parse` | URI shape validation |
| `test_port_traits_sealed` | external crates cannot impl sealed traits |
| `test_no_credential_byte_in_kernel` | grep crate src for credential-shaped patterns; 0 hits |

## Halt Conditions

- BNF v4.1 naming violation.
- Any port trait introduces business logic.
- Any I/O reachable from kernel.
- Any credential-shaped type that could leak (e.g., Serialize on credential-bearing types).

## Next IP

[`IP-002-router-domain.md`](IP-002-router-domain.md)
