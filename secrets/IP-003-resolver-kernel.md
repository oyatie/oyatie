---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-003-resolver-kernel
status: pending
execution_unit: ChangeSet
owner: axis-cloud-secrets
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, port-location, layer-correctness, data-class]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: oya-cloud-secrets-secret-reference-resolver-kernel

## Intent

Scaffold the kernel crate: port traits (sealed) + entity types + value objects + error types. Zero I/O. Zero business logic. Foundation for every other resolver layer.

## ChangeSet boundary

One new Rust crate at `microservices/cloud-secrets/src/crates/oya-cloud-secrets-secret-reference-resolver-kernel/`. Workspace member.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-secrets/src/crates/oya-cloud-secrets-secret-reference-resolver-kernel/Cargo.toml` | create |
| `…/src/lib.rs` | create — module surface + `pub use` |
| `…/src/entities.rs` | create — `SecretReference`, `ResolvedSecret`, `CacheEntry`, `RevocationEvent`, `DataClass` |
| `…/src/ports.rs` | create — sealed traits: `OpenBaoClient`, `SecretCache`, `RevocationConsumer` |
| `…/src/errors.rs` | create — `KernelError`, `ResolveError`, `CacheError`, `RevocationError` |
| `Cargo.toml` (workspace) | update — add member |
| `secrets/catalog/oya-cloud-secrets-secret-reference-resolver-kernel.yaml` | create |

## Crate Naming Justification

```
NAME: oya-cloud-secrets-secret-reference-resolver-kernel
- microservice = cloud-secrets (ADR-0131)
- bc-tokens = secret-reference-resolver (primary BC; ADR-0056 v4.1)
- layer = kernel (ADR-0105 13-value enum; inner/pure; port traits + entities only)
- exemptions: none
```

## Code Shape

```rust
// src/lib.rs
pub mod entities;
pub mod errors;
pub mod ports;
pub use entities::*;
pub use errors::*;
pub use ports::*;

#[doc(hidden)]
mod sealed { pub trait Sealed {} }
```

```rust
// src/entities.rs (excerpt)
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecretReference {
    #[data_class(INTERNAL_ONLY)]
    pub tenant: TenantHandle,
    #[data_class(INTERNAL_ONLY)]
    pub microservice: String,
    #[data_class(INTERNAL_ONLY)]
    pub name: String,
    #[data_class(INTERNAL_ONLY)]
    pub version: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TenantHandle {
    Tenant(String),     // sha256 hash; 16 hex chars
    Shared,
}

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct ResolvedSecret {
    #[data_class(SECRET)]
    pub value: Vec<u8>,
    #[data_class(INTERNAL_ONLY)]
    pub version: u64,
    #[data_class(INTERNAL_ONLY)]
    pub data_class: DataClass,
    #[data_class(AUDIT)]
    pub resolved_at: chrono::DateTime<chrono::Utc>,
    #[data_class(AUDIT)]
    pub integrity_hmac: Vec<u8>,
}

// Explicit Debug to redact value
impl std::fmt::Debug for ResolvedSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ResolvedSecret")
            .field("value", &"[REDACTED]")
            .field("version", &self.version)
            .field("data_class", &self.data_class)
            .field("resolved_at", &self.resolved_at)
            .finish()
    }
}
```

```rust
// src/ports.rs
use async_trait::async_trait;
use crate::sealed::Sealed;
use crate::entities::*;
use crate::errors::*;

#[async_trait]
pub trait OpenBaoClient: Send + Sync + Sealed {
    async fn read(&self, reference: &SecretReference) -> Result<ResolvedSecret, ResolveError>;
    async fn write(&self, reference: &SecretReference, value: Vec<u8>) -> Result<u64, ResolveError>;
    async fn revoke(&self, reference: &SecretReference) -> Result<(), ResolveError>;
}

#[async_trait]
pub trait SecretCache: Send + Sync + Sealed {
    async fn get(&self, key: &SecretReference) -> Option<ResolvedSecret>;
    async fn put(&self, key: SecretReference, value: ResolvedSecret, ttl: std::time::Duration);
    async fn invalidate(&self, key: &SecretReference);
}

#[async_trait]
pub trait RevocationConsumer: Send + Sync + Sealed {
    async fn next(&mut self) -> Result<RevocationEvent, RevocationError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-cloud-secrets-secret-reference-resolver-kernel --all-features
cargo clippy -p oya-cloud-secrets-secret-reference-resolver-kernel --all-features -- -D warnings
cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-kernel --all-features
cargo deny check
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-cloud-secrets-secret-reference-resolver-kernel
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-cloud-secrets-secret-reference-resolver-kernel
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-cloud-secrets-secret-reference-resolver-kernel
```

## Test Plan

Kernel class: 90% line / 80% branch.
- `test_secret_reference_construction`
- `test_resolved_secret_debug_redacts_value`
- `test_resolved_secret_drop_zeroises`
- `test_port_traits_sealed`
- `test_data_class_annotations_present`

## Halt Conditions

- Any I/O reachable from kernel — refactor.
- `Debug` impl on ResolvedSecret leaks value — BLOCKER.

## Next IP

`IP-004-resolver-domain.md`

## References

- ADR-0105, ADR-0106, ADR-0028 (data-class)
- `secrets/contracts/proto/cloud-secrets.proto`

## Wave 15-IP-substance counterpart anchor

Preserved as substantive: this IP already defines concrete kernel entities, ports, zeroization expectations, and crate targets for `oya-cloud-secrets-secret-reference-resolver-kernel`. Counterpart evidence comes from the parity matrices: AWS Secrets Manager, Google Secret Manager, Azure Key Vault, HashiCorp Vault, and Akeyless all expose SDK retrieval primitives, but Oyatie's differentiator is a kernel-level `Secret<T>` and port boundary that makes raw-value logging, unbounded TTL, and unaudited revocation impossible for downstream adapters to normalize away.

Grep-recognized counterpart anchor: GitHub Actions Secrets is relevant only at the CI boundary where resolver tests and branch gates consume secret handles. This kernel IP keeps that distribution concern outside the primary comparator truth, which remains Vault/OpenBao/KMS resolver behavior.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `secrets/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `secrets/contracts/openapi/cloud-secrets.yaml`, `secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `secrets/contracts/proto/cloud-secrets.proto`, `secrets/IP-003-resolver-kernel.md`.
