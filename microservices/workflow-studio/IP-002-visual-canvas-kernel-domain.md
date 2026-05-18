---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
impl_plan_id: IP-002-visual-canvas-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow + council-design-system
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, port-location, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: visual-canvas — kernel + domain crates

## Intent

Author the kernel + domain crates for the visual-canvas BC: pure entity types (Canvas, Node, Edge, Selection, ViewportState), port traits (CanvasStateStore, CanvasEventEmitter), and deterministic layout algebra. Foundation for IP-012 Leptos browser-WASM components.

## ChangeSet boundary

Two crates:
- `oya-workflow-studio-visual-canvas-kernel` — port-trait + entity types; zero I/O; `#![deny(unsafe_code)]`.
- `oya-workflow-studio-visual-canvas-domain` — pure visual layout algebra; deterministic node-placement math; round-trip-stable.

Per ADR-0105 13-layer + ADR-0056 BNF v4.1.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/workflow-studio/src/crates/oya-workflow-studio-visual-canvas-kernel/Cargo.toml` | create | kernel crate |
| `microservices/workflow-studio/src/crates/oya-workflow-studio-visual-canvas-kernel/src/lib.rs` | create | `#![deny(unsafe_code)]` + module structure |
| `microservices/workflow-studio/src/crates/oya-workflow-studio-visual-canvas-kernel/src/entities.rs` | create | `Canvas`, `Node`, `Edge`, `Selection`, `ViewportState` |
| `microservices/workflow-studio/src/crates/oya-workflow-studio-visual-canvas-kernel/src/ports.rs` | create | `CanvasStateStore`, `CanvasEventEmitter` traits |
| `microservices/workflow-studio/src/crates/oya-workflow-studio-visual-canvas-kernel/src/data_class.rs` | create | `#[data_class(BEHAVIORAL_TENANT_PRODUCT)]` proc-macro for fields |
| `microservices/workflow-studio/src/crates/oya-workflow-studio-visual-canvas-domain/Cargo.toml` | create | domain crate |
| `microservices/workflow-studio/src/crates/oya-workflow-studio-visual-canvas-domain/src/lib.rs` | create | imports kernel only |
| `microservices/workflow-studio/src/crates/oya-workflow-studio-visual-canvas-domain/src/layout.rs` | create | deterministic layout algebra |
| `microservices/workflow-studio/src/crates/oya-workflow-studio-visual-canvas-domain/src/selection.rs` | create | selection state transitions |
| `microservices/workflow-studio/src/crates/oya-workflow-studio-visual-canvas-domain/src/viewport.rs` | create | viewport pan/zoom math |
| `microservices/workflow-studio/src/crates/oya-workflow-studio-visual-canvas-domain/tests/layout.rs` | create | property tests for layout determinism |
| `microservices/workflow-studio/catalog/oya-workflow-studio-visual-canvas-kernel.yaml` | create | catalog record |
| `microservices/workflow-studio/catalog/oya-workflow-studio-visual-canvas-domain.yaml` | create | catalog record |

## Crate Naming

Per ADR-0056 v4.1 BNF: `oya-<microservice>-<bc-tokens>-<layer>`.
- `oya-workflow-studio-visual-canvas-kernel`
- `oya-workflow-studio-visual-canvas-domain`

## Code Shape

`kernel/src/entities.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub String);

/// `#[data_class(BEHAVIORAL_TENANT_PRODUCT)]` annotation enforced by
/// `oya-check-data-class` LEAN lane.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Canvas {
    pub canvas_id: String,
    pub tenant_id: String,
    pub definition_id: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub viewport: ViewportState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub node_type: String,
    pub position: (f64, f64),
    pub size: (f64, f64),
    pub config: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub from_node: NodeId,
    pub to_node: NodeId,
    pub from_port: String,
    pub to_port: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Selection {
    pub selected_nodes: Vec<NodeId>,
    pub selected_edges: Vec<EdgeId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewportState {
    pub pan_x: f64,
    pub pan_y: f64,
    pub zoom: f64,
}
```

`kernel/src/ports.rs`:

```rust
use crate::entities::Canvas;
use async_trait::async_trait;

#[async_trait]
pub trait CanvasStateStore: Send + Sync {
    async fn load(&self, canvas_id: &str) -> Result<Canvas, CanvasStoreError>;
    async fn save(&self, canvas: &Canvas) -> Result<(), CanvasStoreError>;
}

#[derive(thiserror::Error, Debug)]
pub enum CanvasStoreError {
    #[error("not found")] NotFound,
    #[error("tenant mismatch")] TenantMismatch,
    #[error(transparent)] Other(#[from] anyhow::Error),
}
```

`domain/src/layout.rs`:

```rust
use oya_workflow_studio_visual_canvas_kernel::entities::*;

/// Deterministic layered layout (Sugiyama-style); round-trip-stable.
pub fn layout_layered(canvas: &Canvas) -> Vec<(NodeId, (f64, f64))> {
    // Pure function; given the same Canvas input, MUST return identical output.
    let mut result: Vec<_> = canvas.nodes.iter().map(|n| (n.id.clone(), n.position)).collect();
    result.sort_by_key(|(id, _)| id.0.clone());
    result
}
```

## Acceptance Gates

```bash
cargo check -p oya-workflow-studio-visual-canvas-kernel -p oya-workflow-studio-visual-canvas-domain
cargo clippy -p oya-workflow-studio-visual-canvas-kernel -p oya-workflow-studio-visual-canvas-domain -- -D warnings
cargo nextest run -p oya-workflow-studio-visual-canvas-kernel -p oya-workflow-studio-visual-canvas-domain
cargo run -p oya-dev-cli -- gate validate lean-a1 --microservice workflow-studio
cargo run -p oya-dev-cli -- gate validate port-location --microservice workflow-studio
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice workflow-studio
cargo run -p oya-dev-cli -- gate validate data-class-annotations --microservice workflow-studio
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_canvas_serde_round_trip` | Canvas serializes + deserializes byte-identically |
| `test_layout_determinism` (property) | layout_layered(canvas) == layout_layered(canvas) for 1000 random canvases |
| `test_layout_invariant_no_overlap` (property) | no two nodes overlap after layout |
| `test_selection_transitions` | add/remove selection is commutative |
| `test_viewport_pan_zoom_inverse` | pan then unpan returns identical viewport |

## Halt Conditions

- Property tests fail — bug; do not mask.
- Layer-correctness lane fails (kernel imports domain or adapter) — restructure.
- Data-class annotation lane fails — add missing annotations.

## Next IP

[`IP-003-dsl-emitter-loader-kernel-domain.md`](IP-003-dsl-emitter-loader-kernel-domain.md)

## References

- ADR-0056 BNF v4.1.
- ADR-0105 13-layer enum.
- ADR-0028 audit-chain + data-class.
- PRD §"Bounded Contexts" — visual-canvas.
- Sugiyama et al. — "Methods for visual understanding of hierarchical system structures" (1981).
