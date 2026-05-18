---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-preview
phase: P01-slides-foundation
impl_plan_id: IP-014-visual-canvas-leptos-wasm-rest-sdk-app
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workspace + council-design-system
acceptance_lanes: [cargo-check, cargo-nextest, wasm-bundle-sri, present-mode-frame-budget, tti-budget]
depends_on: [IP-008, IP-009, IP-010, IP-012, IP-013]
---

# IP-014: Composition: rest + app + adapter-leptos-wasm wiring

## Intent

Author the composition root: per-BC REST adapters, the slides-presentation app binary (Axum + Leptos SSR), all `-adapter-leptos-wasm` crates per ADR-SLIDES-0002, the cargo-leptos build pipeline with SRI per AC-12, and per-route bundle splitting.

## ChangeSet boundary

~30 crates spanning the `-rest` + `-app` + `-adapter-leptos-wasm` layer family.

## Concrete File Targets

`src/crates/oya-slides-presentation-{rest,app}`, `oya-slides-slide-adapter-leptos-wasm`, `oya-slides-text-box-adapter-leptos-wasm`, `oya-slides-shape-adapter-leptos-wasm`, `oya-slides-table-adapter-leptos-wasm`, `oya-slides-animations-adapter-leptos-wasm`, `oya-slides-transitions-adapter-leptos-wasm`, `oya-slides-slide-sorter-adapter-leptos-wasm`, `oya-slides-master-slide-editor-adapter-leptos-wasm`, `oya-slides-presenter-view-adapter-leptos-wasm`, `oya-slides-audience-view-adapter-leptos-wasm`.

Plus the optional canvas-2d tier `oya-slides-slide-adapter-leptos-wasm-canvas2d` (engaged on heuristic threshold per ADR-SLIDES-0002).

## Code Shape

`slide-adapter-leptos-wasm/src/lib.rs`:

```rust
use leptos::*;

#[component]
pub fn SlideView(slide: ReadSignal<Slide>) -> impl IntoView {
    view! {
        <svg
            role="img"
            aria-label=move || format!("Slide {}", slide().ordinal)
            class="oya-slides-slide-svg"
        >
            <For
                each=move || slide().placeholders.clone()
                key=|p| p.placeholder_id.clone()
                children=|p| view! { <PlaceholderView placeholder=p /> }
            />
        </svg>
    }
}
```

`presentation-app/src/main.rs`:

```rust
fn main() {
    // SSR for editor-shell route; CSR for canvas + present-mode.
    let routes = generate_route_list(App);
    let app = create_axum_router(routes, App);
    serve(app).await
}
```

## Acceptance Gates

```bash
cargo build --target wasm32-unknown-unknown -p oya-slides-slide-adapter-leptos-wasm
cargo-leptos build --release
cargo nextest run -p oya-slides-slide-adapter-leptos-wasm --test sri
cargo nextest run -p oya-slides-slide-adapter-leptos-wasm --test wasm_render
oya gate validate wasm-bundle-sri --microservice slides
oya gate validate present-mode-frame-budget --microservice slides
tests/load/tti-budget.js  # Lighthouse synthetic
```

## Halt Conditions

- WASM SRI mismatch — STOP.
- Present-mode 50-slide deck p95 > 50ms transition — STOP. AC-09 invariant.
- TTI cold > 400ms p95 — STOP.

## Next IP

IP-015.
