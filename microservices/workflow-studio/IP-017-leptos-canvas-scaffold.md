---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: workflow-studio
milestone: M03-studio-preview
phase: P02-native-canvas-shells
impl_plan_id: IP-017-leptos-canvas-scaffold
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-frontend
co_owners: [axis-perf]
date: 2026-05-18
related_adrs: [ADR-0185, ADR-0204]
acceptance_lanes: [perf-canvas-60fps-leptos, a11y-axe-zero-violations, oya-vcs-promotion-readiness]
depends_on: [IP-016]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-017 — Leptos canvas scaffold (Phase 2 in-house oya-canvas-leptos)

## Goal

Scaffold `oya-workflow-studio-canvas-leptos` — a Rust-native canvas/node-editor crate running on Leptos signals + SVG/Canvas2D primitives, with a WebGL escape hatch behind a feature flag for >5k node graphs. Per ADR-0204 Phase 2 path: Leptos web ships per ADR-0185 first; this scaffold is the substrate that replaces `@xyflow/svelte` (IP-016 Phase 1) once Phase 2 trigger fires. Adapter contract from IP-016 is honored so the swap is bounded blast radius.

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `microservices/workflow-studio/src/canvas_leptos/Cargo.toml` | create | ~50 LoC; leptos, web-sys, gloo-events, wgpu (gated) |
| `microservices/workflow-studio/src/canvas_leptos/src/lib.rs` | create | ~120 LoC; crate root + public surface |
| `microservices/workflow-studio/src/canvas_leptos/src/canvas.rs` | create | ~300 LoC; `Canvas<F>` Leptos component; viewport signals; LOD ladder |
| `microservices/workflow-studio/src/canvas_leptos/src/virtualization.rs` | create | ~180 LoC; element pool + virtual list; reuses DOM nodes across pan/zoom |
| `microservices/workflow-studio/src/canvas_leptos/src/lod.rs` | create | ~80 LoC; 3-tier LOD: full / simplified / rect-only thresholds |
| `microservices/workflow-studio/src/canvas_leptos/src/webgl_escape.rs` | create | ~220 LoC; gated by `feature = "webgl-escape-hatch"`; wgpu canvas renderer for >5k nodes |
| `microservices/workflow-studio/src/canvas_leptos/src/a11y.rs` | create | ~100 LoC; ARIA tree role + keyboard nav; live region |
| `microservices/workflow-studio/src/canvas_leptos/src/adapter.rs` | create | ~120 LoC; impl `CanvasAdapter` trait matching IP-016 |
| `microservices/workflow-studio/src/canvas_leptos/tests/virtualization_test.rs` | create | ~140 LoC; 3 tests |
| `microservices/workflow-studio/src/canvas_leptos/tests/lod_test.rs` | create | ~80 LoC; 2 tests |
| `microservices/workflow-studio/src/canvas_leptos/tests/adapter_test.rs` | create | ~160 LoC; 3 tests |
| `microservices/workflow-studio/src/canvas_leptos/tests/perf_test.rs` | create | ~120 LoC; 2 perf tests |
| `microservices/workflow-studio/runbooks/leptos-canvas-debug.md` | create | ~80 LoC playbook |
| `microservices/workflow-studio/decisions/ADR-0204.md` | append §"Leptos scaffold landed" | +6 LoC |

## Code shape

`canvas_leptos/src/canvas.rs` (excerpt):

```rust
#[component]
pub fn Canvas(viewport: RwSignal<Viewport>, nodes: ReadSignal<Vec<Node>>, edges: ReadSignal<Vec<Edge>>) -> impl IntoView {
    let lod = Memo::new(move |_| LodLadder::resolve(viewport.get().zoom));
    let visible = Memo::new(move |_| virtualize_visible(viewport.get(), &nodes.get()));
    view! {
        <svg role="tree" aria-label="Workflow canvas" class="oya-canvas">
            <For each=visible key=|n| n.id let:node>
                <NodeView node=node lod=lod.get() />
            </For>
        </svg>
    }
}
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `virtualization_pool_reuses_dom_nodes` | virtualization_test.rs | After 100 pan ops, ≤ 1.1× max-visible nodes ever instantiated |
| `virtualization_visible_set_correct_at_zoom_boundaries` | virtualization_test.rs | At each LOD tier boundary, visible set matches expected geometry |
| `virtualization_no_leak_after_unmount` | virtualization_test.rs | After component unmount, pool drains to zero (Drop semantics) |
| `lod_full_tier_below_25pct_zoom` | lod_test.rs | Zoom < 0.25 → rect-only tier |
| `lod_simplified_tier_25_to_75pct` | lod_test.rs | 0.25 ≤ zoom < 0.75 → simplified tier |
| `adapter_matches_ip_016_canvas_adapter_contract` | adapter_test.rs | All methods present + signatures match |
| `adapter_render_node_emits_aria_tree_item` | adapter_test.rs | Rendered node has `role="treeitem"` |
| `adapter_keyboard_nav_tab_cycles_nodes` | adapter_test.rs | Tab cycles through node items |
| `perf_1000_node_leptos_p99_under_16_67ms` | perf_test.rs | p99 frame ≤ 16.67ms |
| `perf_5000_node_with_webgl_p99_under_16_67ms` | perf_test.rs | Gated on `webgl-escape-hatch`; p99 ≤ 16.67ms |

Minimum 5 required; 10 specified.

## Phase 2 trigger (ADR-0204)

ADR-0204 fixes Phase 2 trigger at "Leptos web ships per ADR-0185" **AND** ">5k node graph becomes the median". Until then, this crate stays Drafting + scaffolded-only; IP-016 (svelte-flow) remains the production path. After Phase 2 trigger fires, a separate cutover IP (TBD-numbered) executes the swap with feature-flag rollout per docs/standards/cutover-and-rollout.md.

## Evidence to emit

- `evidence/microservices/workflow-studio/leptos-canvas-correctness-{date}.json`
- `evidence/microservices/workflow-studio/leptos-canvas-perf-{date}.json`
- Audit-chain seal: `oya audit-chain seal --kind canvas-substrate-scaffold --crate oya-workflow-studio-canvas-leptos --window 30d`
- Metrics: `oya_workflow_studio_canvas_leptos_pool_active`, `oya_workflow_studio_canvas_leptos_frame_time_ms_bucket`

## Rollback procedure

1. Revert ChangeSet for `microservices/workflow-studio/src/canvas_leptos/`.
2. Remove crate from workspace `Cargo.toml`.
3. Phase 2 cutover blocked until scaffold restored.
4. Emit rollback evidence JSON.

## Blocking dependencies

- IP-016 — adapter contract.
- ADR-0185 — client stack.
- ADR-0204 — canvas perf bar.

## Acceptance gates

```bash
cargo run -p oya-dev-cli -- gate validate perf-canvas-60fps-leptos
cargo run -p oya-dev-cli -- gate validate a11y-axe-zero-violations --target canvas-leptos
cargo run -p oya-dev-cli -- gate validate oya-vcs-promotion-readiness --microservice workflow-studio
cargo test -p oya-workflow-studio-canvas-leptos --tests
```

## Halt conditions

- 1000-node p99 > 16.67ms: STOP (scaffold fails perf bar before Phase 2 fires).
- Adapter contract drift from IP-016: STOP — blast-radius contract violated.
- A11y axe-core regression: STOP.

## Exit criteria

1. All 10 tests green on CI (Linux x86_64 + arm64).
2. WASM bundle ≤ 600KiB gzipped (CDN budget; smaller than svelte-flow path).
3. `perf-canvas-60fps-leptos`, `a11y-axe-zero-violations`, `oya-vcs-promotion-readiness` lanes green.
4. Evidence ledger sealed.
5. Runbook published.
6. ADR-0204 Phase 2 substrate section updated.

## Next IP

[`IP-018-swiftui-canvas-impl.md`](IP-018-swiftui-canvas-impl.md)

## References

- ADR-0185 — client stack matrix.
- ADR-0204 — canvas perf bar + Phase 2 trigger.
- IP-016 — svelte-flow (Phase 1).
- Leptos book — `https://leptos-rs.github.io/leptos/`.
- wgpu — `https://wgpu.rs/`.
