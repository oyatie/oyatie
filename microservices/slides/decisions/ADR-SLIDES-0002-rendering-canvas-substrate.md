---
id: ADR-SLIDES-0002
title: Rendering canvas substrate for the slides editor + present-mode
microservice: slides
status: Accepted
date: 2026-05-17
owner: axis-workspace + council-design-system
deciders: council-design-system, council-architecture, axis-workspace, ops-sre-reliability, ops-accessibility
supersedes: []
superseded_by: []
related: [ADR-0065, ADR-0105, ADR-0126, ADR-0131, ADR-WS-0003]
related_specs: [/specs/per-microservice-flat-layout.json]
related_artifacts:
  - microservices/slides/PRD.md (FR-15, AC-09, AC-17, §"Performance")
  - microservices/slides/PHASE-01-SLIDES-FOUNDATION.md (IP-014)
  - microservices/slides/decisions/ADR-SLIDES-0004-animation-engine-and-reduced-motion.md
  - microservices/workflow-studio/decisions/ADR-WS-0003-leptos-wasm-substrate.md
purpose: Establish Leptos WASM + SVG-baseline + canvas-2d + WebGL-fallback as the slides canvas + present-mode rendering substrate, with the 60fps present-mode transition invariant as the load-bearing performance property.
doc_status: published
---

# ADR-SLIDES-0002: Rendering canvas substrate — Leptos WASM + SVG-baseline (with canvas-2d + WebGL fallback for present-mode)

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The slides editor + present-mode is one of the largest Leptos applications in oyatie (alongside workflow-studio per ADR-WS-0003 and the workspace docs / sheets siblings). It must support:

- **Editor authoring**: drag-drop placeholders, vector shapes, text-boxes, images, video/audio embeds, charts, tables, equations. Slide-sorter view with thumbnails. Master-slide editor. Theme/template gallery.
- **Present-mode at 60fps**: slide-to-slide transition p95 ≤ 50ms; per-frame budget p99 ≤ 16.7ms (60fps invariant) over 50-slide deck.
- **Animation engine**: entrance/emphasis/exit/path animations; transitions (fade, slide, push, morph). Reduced-motion fallback (ADR-SLIDES-0004) compliant with WCAG 2.2 SC 2.3.3.
- **Broadcast-mode**: read-only audience-view rendering with reactions/polls/Q&A overlay (ADR-SLIDES-0005).
- **Accessibility**: WCAG 2.2 AA + ARIA roles per canvas primitive + keyboard-only authoring + color-blind-safe palette validation.

ADR-0065 (repo-wide) settled Leptos as oyatie's browser UI substrate; ADR-WS-0003 (workflow-studio) settled Leptos 0.7+ + SVG canvas + signals + cargo-leptos for the workflow-studio canvas, which is the closest sibling.

The slides canvas differs from the workflow-studio canvas in qualitatively important ways:

- **Render workload spike during present-mode**: workflow-studio canvas is editor-only; slides has BOTH an editor AND a present-mode surface. Present-mode is read-only but with 60fps animation rendering — a stricter frame budget than editor drag interactions.
- **Vector + raster mix**: slides renders shapes (vector), images (raster), videos (raster + audio), charts (vector + data), tables (vector + DOM-like text grid), equations (KaTeX/MathJax output). Workflow-studio is primarily vector (nodes + edges).
- **Animation timing model**: slides requires deterministic animation timing for present-mode replay + MP4 export determinism (ADR-SLIDES-0003). Workflow-studio has no animation surface.
- **Per-slide layered rendering**: slides composes multiple layers (background + placeholders + animations + audience-engagement overlay). Workflow-studio composes one layer.

This ADR resolves PRD Open Question 1 (Vector rendering tier — SVG baseline vs canvas-2d vs WebGL).

## Decision

Adopt **Leptos 0.7+ + signals + cargo-leptos** as the slides canvas + present-mode substrate, in the following shape:

1. **Compile target**: `wasm32-unknown-unknown` for the browser bundle; cargo-leptos orchestrates the build. The slides canvas runs entirely in the browser (CSR mode for the canvas). SSR enabled for the editor-shell route (login redirect, tenant resolution, OIDC bootstrap, deck-list); SSR-rendering a 50-slide canvas offers no observable benefit.
2. **Reactivity model**: Leptos signals (`Signal<T>`, `ReadSignal<T>`, `WriteSignal<T>`, `Memo<T>`) for all reactive state. Fine-grained reactivity is the load-bearing performance property; only slides/placeholders whose signals changed re-render.
3. **Three-tier rendering pipeline**:
   - **Tier 1 — SVG baseline** for editor authoring (drag, click, hover, keyboard nav, ARIA roles per primitive); 95% of editor surface. SVG is accessibility-friendly + DOM-debug-friendly + CSS-theme-able.
   - **Tier 2 — canvas-2d for present-mode** when frame budget at SVG fails measurement: an `-adapter-leptos-wasm-canvas2d` crate renders the 60fps present-mode using `web-sys` Canvas 2D API. Engaged automatically when device pixel ratio × deck complexity exceeds heuristic threshold.
   - **Tier 3 — WebGL fallback** for very-complex decks (50+ slides with rich animations + video-embed). An `-adapter-leptos-wasm-webgl` crate using `web-sys` WebGL2 API + tiny-skia-style draw primitives. Engaged only after Tier-2 fails measurement; post-M03 work.
4. **No external JS canvas library** (e.g., Konva, PixiJS, Fabric). Mixing JS canvas with Rust signals doubles failure modes (T-T-02 SRI scope + CSP scope + WASM↔JS bridge cost per frame).
5. **Bundle splitting**: per-route code splitting via cargo-leptos. Editor, present-mode, slide-sorter, master-slide-editor, audience-view shipped as separate WASM chunks; each chunk carries SHA-384 SRI hash per AC-12.
6. **WASM↔JS interop**: minimized via `wasm-bindgen`. JS interop limited to (a) browser APIs not yet bound in `web-sys`, (b) WebSocket via `gloo-net`, (c) LiveKit client via the messenger SDK's WASM bindings, (d) WASM bootstrap nonce script.
7. **Test substrate**: `wasm-bindgen-test` for component tests; `playwright-rust` for e2e; Lighthouse-style synthetic for TTI assertion per AC-09; per-frame timing assertion via `requestAnimationFrame` timestamp logging.
8. **Accessibility**: WCAG 2.2 AA target; ARIA roles per canvas primitive (svg `role="img"` + `aria-label`); keyboard navigation; reduced-motion via ADR-SLIDES-0004.
9. **Present-mode 60fps verification**: an in-build `oya-governance-present-mode-frame-budget` lane runs the 50-slide golden deck under headless Chrome; asserts p95 transition ≤ 50ms + p99 frame ≤ 16.7ms.

## Alternatives Considered

### Alternative A — React + canvas-2d (TypeScript)

Industry-default.

- **Pros**
  - Largest hiring pool + ecosystem.
  - PixiJS / Konva / Fabric are mature canvas-2d libraries.
- **Cons**
  - VDOM diff cost scales with tree size; 50-slide editor + present-mode hits this hard.
  - No code-share with Rust-native `-domain` crates (Loro projection, layout-engine, animation-engine, equation-rendering would need to be re-authored in TS or invoked through WASM↔JS bridge per signal change).
  - Bundle size for React + canvas library ~300-500 KB gzip before app code.
  - Forks slides' developer-experience surface from cross-µservice Rust majority.
- **Rejected reason**: code-share + bridge-cost + bundle-size dominate; ADR-WS-0003 alternatives analysis already applies; we extend rather than re-litigate.

### Alternative B — SolidJS

Fine-grained reactive framework, closest JS-side analogue to Leptos.

- **Pros**: Fine-grained signals match Leptos's reactivity. Smaller bundle than React.
- **Cons**: Same code-share problem (no Rust). Smaller ecosystem.
- **Rejected reason**: Code-share problem; bridge cost.

### Alternative C — Svelte 5 / SvelteKit

Compile-time reactive framework with runes.

- **Pros**: Compile-time reactivity; small runtime.
- **Cons**: Same code-share problem; canvas libraries scarce; Svelte 5 production-stability at large scale not yet demonstrated.
- **Rejected reason**: Code-share dominates.

### Alternative D — Yew (Rust browser-WASM)

React-style Rust-to-WASM with VDOM.

- **Pros**: Native Rust; code-share with `-domain` crates.
- **Cons**: VDOM-based — same scale-out problem as React for 60fps present-mode. Smaller ecosystem than Leptos. SSR less mature.
- **Rejected reason**: VDOM frame-budget concern.

### Alternative E — Dioxus

Another Rust-to-WASM framework (cross-platform).

- **Pros**: Native Rust + cross-platform.
- **Cons**: VDOM-based; multi-target ambition spreads engineering effort.
- **Rejected reason**: VDOM + multi-target.

### Alternative F — Pure SVG without canvas-2d/WebGL tier

Stay with SVG only.

- **Pros**: Simpler; single rendering pipeline.
- **Cons**: SVG hits frame-budget ceiling around 30-40 slides at 60fps on commodity hardware; present-mode mid-tier laptops fall below 60fps target.
- **Rejected reason**: Cannot defend AC-09 60fps invariant on commodity hardware without canvas-2d tier.

### Alternative G — Pure canvas-2d (no SVG layer)

Render everything via canvas-2d.

- **Pros**: Single rendering pipeline; consistent frame budget; works at scale.
- **Cons**: Canvas-2d has no built-in accessibility — every element must manually emit ARIA via shadow DOM. CSS theming becomes manual. DOM-inspector debugging painful. Authoring (drag, hover, click) requires manual hit-testing.
- **Rejected reason**: Accessibility (AC-17 + WCAG 2.2 AA) cost is too high; authoring UX in canvas-2d is significantly worse than SVG.

### Alternative H — Pure WebGL

Render everything via WebGL.

- **Pros**: Maximum performance; gaming-grade animations possible.
- **Cons**: Same accessibility problem as canvas-2d, only worse. Browser compatibility (mobile) less reliable. CSS theming impossible without bespoke shaders. Significant engineering investment.
- **Rejected reason**: Premature optimization; accessibility cost; engineering cost.

## Consequences

### Architectural

- The visual-canvas-tier crates `oya-slides-slide-adapter-leptos-wasm`, `oya-slides-text-box-adapter-leptos-wasm`, `oya-slides-shape-adapter-leptos-wasm`, `oya-slides-animations-adapter-leptos-wasm`, `oya-slides-transitions-adapter-leptos-wasm`, `oya-slides-table-adapter-leptos-wasm`, `oya-slides-slide-sorter-adapter-leptos-wasm`, `oya-slides-master-slide-editor-adapter-leptos-wasm`, `oya-slides-presenter-view-adapter-leptos-wasm`, `oya-slides-audience-view-adapter-leptos-wasm` are SVG-baseline.
- The present-mode `-adapter-leptos-wasm-canvas2d` crate kicks in on heuristic threshold; tested under headless Chrome on 50-slide golden deck.
- The WebGL `-adapter-leptos-wasm-webgl` crate is a non-blocking exploration; engaged only post-M03 if canvas-2d hits a measured ceiling.
- The `-domain` crates (visual-layout algebra, animation timing, equation rendering, deck composition) are pure Rust + signal-driven + WASM-target-parity tested.
- The `-app` composition root emits SSR + WASM via cargo-leptos.
- WASM bundle SHA-384 SRI per chunk; verified at every PR via `oya-governance-wasm-bundle-sri` lane.

### Downstream impact on other µservices and IPs

1. **IP-002 (presentation + slide kernel + domain)** — Leptos-agnostic; pure Rust. No change.
2. **IP-014 (composition: rest + app + adapter-leptos-wasm)** — Leptos-specific implementation; cargo-leptos build pipeline; SRI lane wired here.
3. **All Slides BCs with `-adapter-leptos-wasm` layer** — adopt the Leptos signals + `view!` macro + keyed `<For>` patterns.
4. **workflow-engine + sheets + docs µservices** — unaffected at server layer; share `application` µservice's hosting shell.
5. **observability µservice** — slides-specific Lighthouse synthetic + per-frame timing SLI emission.
6. **cloud-iac µservice** — CDN edge cache for slides WASM chunks (Cache-Control: immutable + SHA-384 SRI; long TTL).

### SLOs and CI lanes affected

- `oya-governance-wasm-bundle-sri` — BLOCKER lane on dev + staging.
- `oya-governance-present-mode-frame-budget` — NEW lane; asserts 50-slide golden deck p95 transition ≤ 50ms + p99 frame ≤ 16.7ms.
- `oya-governance-reduced-motion-fallback-mandatory` — BLOCKER lane (per ADR-SLIDES-0004).
- `slides.editor_tti_p99_ms` — Lighthouse synthetic; target ≤ 400ms cold p95.
- `slides.present_transition_p95_seconds` — Lighthouse + per-frame timing; target ≤ 0.05.
- `slides.present_frame_time_p99_seconds` — target ≤ 0.0167.
- `slides.wasm_bundle_size_bytes` — per-chunk + total; release gate alarms on >+50 KB gzip delta.

### Toolchain + supply-chain

- `cargo-leptos` pinned in workspace toolchain.
- `wasm-bindgen` + `web-sys` + `gloo-net` + `leptos` + `leptos_router` + `leptos_meta` pinned via `Cargo.lock` + cargo-deny review.
- WASM-bundle SRI computed using SHA-384 per W3C SRI; embedded into HTML at SSR.
- Per-pack CDN cache keys per PRD §"Security".

### Risk register

- **Risk**: Leptos 0.7+ breaking changes between minor versions. **Mitigation**: pinning + bi-monthly upstream PR review; coordinated migrations across the Leptos-substrate µservices (workflow-studio + slides + docs + sheets).
- **Risk**: SVG-tier frame-budget ceiling on low-end hardware. **Mitigation**: canvas-2d tier auto-engages; WebGL tier post-M03 if canvas-2d hits ceiling.
- **Risk**: cargo-leptos toolchain instability. **Mitigation**: vendored fallback (raw cargo + wasm-bindgen-cli).
- **Risk**: WebGL fallback never engaged (premature complexity). **Mitigation**: kept under `src/crates/oya-slides-slide-adapter-leptos-wasm-webgl/` as non-blocking exploration; activates only on measurement.

## References

- PRD `microservices/slides/PRD.md` §"Performance", AC-09, AC-12, AC-17.
- ADR-WS-0003 (parent — workflow-studio Leptos substrate).
- ADR-0065 (repo-wide Leptos).
- ADR-0105 (13-layer + adapter-leptos-wasm + backend-qualified Amd.3).
- Leptos — `leptos.dev`, `github.com/leptos-rs/leptos`.
- cargo-leptos — `github.com/leptos-rs/cargo-leptos`.
- W3C Subresource Integrity — `www.w3.org/TR/SRI/`.
- WCAG 2.2 — `www.w3.org/TR/WCAG22/`.
- MDN Canvas 2D — `developer.mozilla.org/docs/Web/API/Canvas_API`.
- MDN WebGL2 — `developer.mozilla.org/docs/Web/API/WebGL2RenderingContext`.
- W3C Media Queries Level 5 `prefers-reduced-motion` — `www.w3.org/TR/mediaqueries-5/`.
- ADR-SLIDES-0001 (Loro CRDT — interaction surface with reactive canvas).
- ADR-SLIDES-0004 (animation engine + reduced-motion).
