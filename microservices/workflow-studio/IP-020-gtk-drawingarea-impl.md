---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: workflow-studio
milestone: M03-studio-preview
phase: P02-native-canvas-shells
impl_plan_id: IP-020-gtk-drawingarea-impl
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-frontend
co_owners: [axis-platform-linux]
date: 2026-05-18
related_adrs: [ADR-0185, ADR-0204, ADR-0207]
acceptance_lanes: [a11y-at-spi, perf-canvas-60fps, oya-vcs-promotion-readiness]
depends_on: [IP-016, IP-022, IP-023]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-020 — GTK 4 DrawingArea canvas impl (Linux native shell)

## Goal

Ship the GTK 4 native canvas for the Workflow Studio Linux shell. Uses `gtk4-rs` bindings backed by a `GtkDrawingArea` with Cairo rendering for nodes/edges, AT-SPI accessibility roles per ADR-0207, RTL via `gtk_widget_set_direction`, and Loro CRDT sync (ADR-0145) via the shared Rust Loro client used in IP-022. The canvas adapter contract from IP-016 is honored so the GTK shell renders the same node-graph model as web/SwiftUI/Compose/WinUI.

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `clients/gtk4/workflow-studio/Cargo.toml` | create | full (~60 LoC; gtk4 + cairo + loro + tokio deps) |
| `clients/gtk4/workflow-studio/src/main.rs` | create | full (~80 LoC; gtk Application bootstrap, init Loro client, attach window) |
| `clients/gtk4/workflow-studio/src/canvas.rs` | create | full (~320 LoC; `CanvasWidget` wraps `GtkDrawingArea`; `draw_func` renders viewport via Cairo; pan/zoom gesture controllers; LOD ladder mirrors IP-016 SvelteFlowCanvasAdapter rules) |
| `clients/gtk4/workflow-studio/src/canvas_adapter.rs` | create | ~120 LoC; impl `CanvasAdapter` trait from `oya-workflow-studio-canvas-contract` matching IP-016 interface |
| `clients/gtk4/workflow-studio/src/a11y.rs` | create | ~140 LoC; AT-SPI role assignment (`accessible.set_role(AccessibleRole::Tree)`, per-node `Item` children, live region for drag state per ADR-0207) |
| `clients/gtk4/workflow-studio/src/rtl.rs` | create | ~40 LoC; reads system locale, applies `set_direction(TextDirection::Rtl)` when needed |
| `clients/gtk4/workflow-studio/src/loro_binding.rs` | create | ~180 LoC; reuses `oya-collab-loro` shared crate (IP-022) and bridges Loro doc updates to `glib::MainContext::default().spawn_local` |
| `clients/gtk4/workflow-studio/src/presence_binding.rs` | create | ~110 LoC; mirrors IP-023 awareness map; renders shared cursors via `cairo_pattern` overlay layer |
| `clients/gtk4/workflow-studio/tests/integration_canvas.rs` | create | ~240 LoC; 6 tests (see below) |
| `clients/gtk4/workflow-studio/tests/integration_a11y.rs` | create | ~120 LoC; AT-SPI conformance + axe-equivalent via `at-spi2-core` |
| `microservices/workflow-studio/runbooks/gtk4-canvas-debug.md` | create | ~80 LoC operator playbook |
| `microservices/workflow-studio/decisions/ADR-0185.md` | append §"GTK 4 shell wired" | +6 LoC status line |

## Code shape

`src/canvas.rs` (excerpt):

```rust
use gtk4::prelude::*;
use gtk4::{DrawingArea, gdk};
use oya_workflow_studio_canvas_contract::{CanvasAdapter, Node, Edge, Viewport};

pub struct GtkCanvas {
    widget: DrawingArea,
    state: Rc<RefCell<CanvasState>>,
}

impl GtkCanvas {
    pub fn new(loro: LoroBinding) -> Self {
        let widget = DrawingArea::builder()
            .hexpand(true).vexpand(true)
            .accessible_role(gtk4::AccessibleRole::Tree)
            .build();
        let state = Rc::new(RefCell::new(CanvasState::new(loro)));
        widget.set_draw_func({
            let state = state.clone();
            move |_, cr, w, h| draw_canvas(cr, w, h, &state.borrow())
        });
        attach_gesture_controllers(&widget, &state);
        Self { widget, state }
    }
}
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `gtk_canvas_renders_1000_nodes_at_60fps` | tests/integration_canvas.rs | p99 frame ≤ 16.67ms over 30s loop |
| `gtk_canvas_pan_zoom_keyboard_nav` | tests/integration_canvas.rs | Arrow keys move viewport; +/- zoom; Tab cycles nodes per ADR-0207 |
| `gtk_canvas_loro_sync_two_peer_merge` | tests/integration_canvas.rs | Two peers edit; CRDT merges; node set equal |
| `gtk_canvas_presence_shared_cursors` | tests/integration_canvas.rs | Peer cursor renders within 200ms p95 |
| `gtk_canvas_rtl_direction_swaps_origin` | tests/integration_canvas.rs | `set_direction(Rtl)` flips origin; AT-SPI exposes `text-direction=rtl` |
| `gtk_canvas_cross_tenant_isolation` | tests/integration_canvas.rs | Connecting with mismatched tenant_id room key → joins empty doc |
| `gtk_canvas_at_spi_conformance` | tests/integration_a11y.rs | `at-spi2-core` introspection finds Tree+Item roles, focused state, live region |
| `gtk_canvas_axe_equivalent_zero_violations` | tests/integration_a11y.rs | WCAG 2.2 AA mapped checks via `Accerciser`-driven probe; 0 violations |

Minimum 5 integration tests required (8 specified above; threshold met with cushion).

## Evidence to emit

- `evidence/microservices/workflow-studio/gtk4-canvas-perf-{date}.json` — frame-time histogram, p50/p95/p99
- `evidence/microservices/workflow-studio/gtk4-canvas-a11y-{date}.json` — AT-SPI conformance probe output
- Audit-chain seal: `oya audit-chain seal --kind canvas-perf --ms workflow-studio --shell gtk4 --window 30d` — emits ledger entry consumed by `perf-budget` lane
- Metrics emitted via OpenTelemetry: `oya_workflow_studio_canvas_frame_time_ms_bucket{shell="gtk4"}`, `oya_workflow_studio_canvas_loro_merge_latency_ms_bucket{shell="gtk4"}`
- Structured logs: `target=clients/gtk4/workflow-studio shell=gtk4 event=canvas.frame.miss frame_ms=...`

## Rollback procedure

1. Revert the `clients/gtk4/workflow-studio/` directory: `git revert <range>` on the IP-020 ChangeSet.
2. Remove crate from workspace `Cargo.toml` members list.
3. Flip feature flag `workflow_studio_shell_gtk4=disabled` via FeatureFlags µservice; SvelteKit web shell remains primary for Linux users.
4. Drop pinned Linux Flatpak build artifacts under `dist/linux/gtk4/` from CDN.
5. Update `microservices/workflow-studio/PRD.md §"Supported shells"` to remove GTK 4 row (table refresh) until re-enabled.
6. Document rollback in `evidence/microservices/workflow-studio/gtk4-rollback-{date}.json` and announce in #workflow-studio-ops.

## Blocking dependencies

- IP-016 (svelte-flow canvas) — defines the canonical `CanvasAdapter` contract this IP must satisfy.
- IP-022 (Loro CRDT sync binding) — supplies the `oya-collab-loro` shared Rust crate this IP consumes.
- IP-023 (presence awareness protocol) — supplies the awareness map shape this IP renders.
- ADR-0207 a11y bar — AT-SPI role mapping table must be authored before tests can assert conformance.

## Acceptance gates

```bash
cargo run -p oya-dev-cli -- gate validate a11y-at-spi --shell gtk4 --crate clients/gtk4/workflow-studio
cargo run -p oya-dev-cli -- gate validate perf-canvas-60fps --shell gtk4 --window 30s
cargo run -p oya-dev-cli -- gate validate oya-vcs-promotion-readiness --microservice workflow-studio
cargo test -p oya-workflow-studio-gtk4 --tests
```

## Halt conditions

- `perf-canvas-60fps` lane fails (p99 > 16.67ms over 30s): STOP, file regression IP, do not promote.
- AT-SPI conformance probe finds any A-level violation: STOP, file a11y-defect IP.
- Loro merge correctness test fails (silent data loss): STOP, escalate to ADR-0145 owner; correctness regression blocks all promotion.

## Exit criteria

1. All 8 integration tests pass on Linux x86_64 + arm64 CI runners.
2. `a11y-at-spi`, `perf-canvas-60fps`, `oya-vcs-promotion-readiness` lanes green.
3. Evidence ledger entries sealed via audit-chain.
4. PRD §"Supported shells" includes GTK 4 row with link to this IP.
5. Runbook `gtk4-canvas-debug.md` linked from `microservices/workflow-studio/runbooks/index.md`.
6. ADR-0185 status updated.

## Next IP

[`IP-021-winui-canvas-impl.md`](IP-021-winui-canvas-impl.md)

## References

- ADR-0185 — client stack matrix.
- ADR-0204 — canvas perf bar.
- ADR-0207 — accessibility bar (AT-SPI mapping table).
- ADR-0145 — Loro CRDT pin.
- ADR-0208 — WebSocket transport.
- gtk4-rs documentation — `https://gtk-rs.org/gtk4-rs/`.
- AT-SPI specification — `https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/`.
