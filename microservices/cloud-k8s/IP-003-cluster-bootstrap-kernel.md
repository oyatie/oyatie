---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-003-cluster-bootstrap-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: oya-cloud-k8s-cluster-bootstrap-kernel

## Intent

Scaffold the `kernel` layer crate per ADR-0105: port traits (sealed) + entity types + value objects + error types. Zero I/O. Zero business logic. Foundation that every other cluster-bootstrap layer crate depends on.

## ChangeSet boundary

One new Rust crate at `microservices/cloud-k8s/src/crates/oya-cloud-k8s-cluster-bootstrap-kernel/`. Workspace member added to root `Cargo.toml`. Catalog row at `microservices/cloud-k8s/catalog/oya-cloud-k8s-cluster-bootstrap-kernel.yaml`.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/cloud-k8s/src/crates/oya-cloud-k8s-cluster-bootstrap-kernel/Cargo.toml` | create | `[package]` + minimal deps |
| `.../src/lib.rs` | create | module declarations + `pub use` surface |
| `.../src/entities.rs` | create | `Cluster`, `ControlPlaneNode`, `KubeadmConfig`, `EtcdSnapshot`, `BootstrapEvidence` with `data_class` annotations |
| `.../src/ports.rs` | create | sealed port traits (KubeadmCommander, EtcdSnapshotter, ControlPlaneInspector) |
| `.../src/errors.rs` | create | error variants per port + entity |
| `Cargo.toml` (workspace) | update | add workspace member |
| `microservices/cloud-k8s/catalog/oya-cloud-k8s-cluster-bootstrap-kernel.yaml` | create | catalog row |

## Crate Naming

```
NAME: oya-cloud-k8s-cluster-bootstrap-kernel
JUSTIFICATION:
- microservice = cloud-k8s
- bc-tokens = cluster-bootstrap
- layer = kernel (ADR-0105 13-value enum: inner/pure; port traits + entities only)
- exemptions claimed: none
```

## Code Shape

```rust
// src/lib.rs
pub mod entities;
pub mod errors;
pub mod ports;

pub use entities::{Cluster, ControlPlaneNode, KubeadmConfig, EtcdSnapshot, BootstrapEvidence};
pub use errors::{KernelError, KubeadmError, EtcdError};
pub use ports::{KubeadmCommander, EtcdSnapshotter, ControlPlaneInspector};

#[doc(hidden)]
mod sealed { pub trait Sealed {} }
```

```rust
// src/entities.rs (excerpt)
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cluster {
    #[data_class(INTERNAL_ONLY)]
    pub cluster_id: String,
    #[data_class(INTERNAL_ONLY)]
    pub pack: Pack,
    #[data_class(INTERNAL_ONLY)]
    pub region: String,
    #[data_class(INTERNAL_ONLY)]
    pub kubeadm_version: String,
    #[data_class(INTERNAL_ONLY)]
    pub control_plane_node_count: u32,
    #[data_class(INTERNAL_ONLY)]
    pub worker_node_count: u32,
    #[data_class(AUDIT)]
    pub status: ClusterStatus,
    #[data_class(AUDIT)]
    pub bootstrapped_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClusterStatus { Bootstrapping, Ready, Upgrading, Degraded, Failed }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Pack {
    Kr, Eu, Us, UsHealthcare, Jp, Sg, Au, In_, Br, Ae, Ksa,
}
```

```rust
// src/ports.rs
use async_trait::async_trait;
use crate::sealed::Sealed;
use crate::entities::*;
use crate::errors::*;

#[async_trait]
pub trait KubeadmCommander: Send + Sync + Sealed {
    async fn init(&self, config: &KubeadmConfig) -> Result<BootstrapEvidence, KubeadmError>;
    async fn upgrade(&self, target_version: &str) -> Result<BootstrapEvidence, KubeadmError>;
    async fn reset(&self) -> Result<(), KubeadmError>;
}

#[async_trait]
pub trait EtcdSnapshotter: Send + Sync + Sealed {
    async fn snapshot(&self) -> Result<EtcdSnapshot, EtcdError>;
    async fn restore(&self, snap: &EtcdSnapshot) -> Result<(), EtcdError>;
    async fn verify_signature(&self, snap: &EtcdSnapshot) -> Result<bool, EtcdError>;
}

#[async_trait]
pub trait ControlPlaneInspector: Send + Sync + Sealed {
    async fn read_cluster(&self, id: &str) -> Result<Cluster, KernelError>;
    async fn list_clusters(&self, pack: Pack) -> Result<Vec<Cluster>, KernelError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-cloud-k8s-cluster-bootstrap-kernel --all-features
cargo build -p oya-cloud-k8s-cluster-bootstrap-kernel --all-features
cargo clippy -p oya-cloud-k8s-cluster-bootstrap-kernel --all-features -- -D warnings
cargo nextest run -p oya-cloud-k8s-cluster-bootstrap-kernel --all-features
cargo deny check
cargo doc -p oya-cloud-k8s-cluster-bootstrap-kernel --no-deps
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-cloud-k8s-cluster-bootstrap-kernel
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-cloud-k8s-cluster-bootstrap-kernel
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-cloud-k8s-cluster-bootstrap-kernel
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-cloud-k8s-cluster-bootstrap-kernel
```

## Test Plan

Per PHASE-01 §"Per-IP Test Coverage Threshold" kernel class: 1 test per public type + 1 per port trait + 1 sealed-trait smoke. Coverage 90% line / 80% branch.

| Test | Verifies |
|---|---|
| `test_cluster_construction` | entity invariants |
| `test_bootstrap_evidence_serde` | serde roundtrip |
| `test_etcd_snapshot_signature_pure` | no I/O |
| `test_port_traits_sealed` | external crates cannot impl sealed traits |
| `test_data_class_annotations_present` | every public field annotated |

## Halt Conditions

- BNF v4.1 naming violation
- Any port introduces business logic — refactor to domain
- Any I/O reachable from kernel — refactor

## Next IP

[`IP-004-cluster-bootstrap-domain.md`](IP-004-cluster-bootstrap-domain.md)

## References

- ADR-0056, ADR-0105, ADR-0106, ADR-0121.
- `microservices/cloud-k8s/PRD.md` §"Bounded Contexts" port-trait table.
- Bominal ADR-0028 (data-class taxonomy).
