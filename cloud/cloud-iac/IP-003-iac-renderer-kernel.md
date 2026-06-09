---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
impl_plan_id: IP-003-iac-renderer-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-iac
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: oya-cloud-iac-iac-renderer-kernel

## Intent

Scaffold the `kernel` layer crate per ADR-0105: port traits + entity types + value objects + error types. Zero I/O. Zero business logic. Foundation that every other iac-renderer layer crate depends on.

## ChangeSet boundary

One new Rust crate at `microservices/cloud-iac/src/crates/oya-cloud-iac-iac-renderer-kernel/`. Workspace member added to root `Cargo.toml`. Catalog row at `microservices/cloud-iac/catalog/oya-cloud-iac-iac-renderer-kernel.yaml`.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/cloud-iac/src/crates/oya-cloud-iac-iac-renderer-kernel/Cargo.toml` | create | `[package]` + minimal deps (`async-trait`, `serde`) |
| `microservices/cloud-iac/src/crates/oya-cloud-iac-iac-renderer-kernel/src/lib.rs` | create | module declarations + `pub use` surface |
| `microservices/cloud-iac/src/crates/oya-cloud-iac-iac-renderer-kernel/src/entities.rs` | create | `ChartSource`, `ModuleSource`, `OverlaySource`, `RenderedManifest`, `ContentDigest` with `data_class` annotations |
| `microservices/cloud-iac/src/crates/oya-cloud-iac-iac-renderer-kernel/src/ports.rs` | create | port trait declarations (sealed): `ChartSourceReader`, `KustomizeOverlayReader`, `TerraformPlanComputer`, `RenderEventEmitter` |
| `microservices/cloud-iac/src/crates/oya-cloud-iac-iac-renderer-kernel/src/errors.rs` | create | error variants per port + entity |
| `Cargo.toml` (workspace) | update | add new crate to `[workspace.members]` |
| `microservices/cloud-iac/catalog/oya-cloud-iac-iac-renderer-kernel.yaml` | create | catalog row |

## Crate Naming

```
NAME: oya-cloud-iac-iac-renderer-kernel
JUSTIFICATION:
- microservice = cloud-iac (microservices/cloud-iac/)
- bc-tokens = iac-renderer (primary BC per PRD §"Bounded Contexts")
- layer = kernel (ADR-0105 13-value enum; inner/pure; port traits + entities only)
- exemptions claimed: none
```

## Code Shape

```rust
// src/lib.rs
pub mod entities;
pub mod errors;
pub mod ports;

pub use entities::{
    ChartSource, ContentDigest, ModuleSource, OverlaySource, RenderedManifest,
};
pub use errors::{KernelError, RenderError};
pub use ports::{
    ChartSourceReader, KustomizeOverlayReader, RenderEventEmitter,
    TerraformPlanComputer,
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
pub struct ChartSource {
    #[data_class(INTERNAL_ONLY)]
    pub microservice: String,
    #[data_class(INTERNAL_ONLY)]
    pub chart_name: String,
    #[data_class(INTERNAL_ONLY)]
    pub version: String,
    #[data_class(INTERNAL_ONLY)]
    pub digest: ContentDigest,
    #[data_class(AUDIT)]
    pub signed_by: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentDigest(pub [u8; 32]);  // sha256
```

```rust
// src/ports.rs
use async_trait::async_trait;
use crate::sealed::Sealed;
use crate::entities::*;
use crate::errors::*;

#[async_trait]
pub trait ChartSourceReader: Send + Sync + Sealed {
    async fn read(&self, microservice: &str, sha: &str) -> Result<Vec<ChartSource>, RenderError>;
}

#[async_trait]
pub trait KustomizeOverlayReader: Send + Sync + Sealed {
    async fn read(&self, microservice: &str, sha: &str, pack: &str) -> Result<Vec<OverlaySource>, RenderError>;
}

#[async_trait]
pub trait TerraformPlanComputer: Send + Sync + Sealed {
    async fn plan(&self, module: &ModuleSource, env: &Environment) -> Result<TerraformPlan, RenderError>;
}

#[async_trait]
pub trait RenderEventEmitter: Send + Sync + Sealed {
    async fn emit_render_completed(&self, manifest: &RenderedManifest) -> Result<(), KernelError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-cloud-iac-iac-renderer-kernel --all-features
cargo build -p oya-cloud-iac-iac-renderer-kernel --all-features
cargo clippy -p oya-cloud-iac-iac-renderer-kernel --all-features -- -D warnings
cargo nextest run -p oya-cloud-iac-iac-renderer-kernel --all-features
cargo deny check
cargo doc -p oya-cloud-iac-iac-renderer-kernel --no-deps
cloud-ci/oya-ci governance gate `lean-a1` for --crate oya-cloud-iac-iac-renderer-kernel is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `port-location` for --crate oya-cloud-iac-iac-renderer-kernel is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `layer-correctness` for --crate oya-cloud-iac-iac-renderer-kernel is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `data-class` for --crate oya-cloud-iac-iac-renderer-kernel is green in the branch-protected `oya-ci-required` context
```

## Test Plan

Per PHASE-01 kernel class: 1 test per public type + 1 per port trait + 1 sealed-trait smoke. Coverage 90% line / 80% branch.

| Test | Verifies |
|---|---|
| `test_chart_source_construction` | entity invariants |
| `test_content_digest_serde` | serde roundtrip; equality |
| `test_rendered_manifest_invariants` | digest correctness |
| `test_port_traits_sealed` | external crates cannot impl sealed traits |
| `test_data_class_annotations_present` | every public field has `#[data_class(..)]` |

## Halt Conditions

- BNF v4.1 naming violation.
- Any port trait introduces business logic.
- Any I/O reachable from kernel.

## Next IP

[`IP-004-iac-renderer-domain-usecase.md`](IP-004-iac-renderer-domain-usecase.md)

## References

- ADR-0056 BNF v4.1; ADR-0105 13-layer enum.
- PRD §"Bounded Contexts" port-trait table.
- Bominal ADR-0028 (data-class taxonomy).
