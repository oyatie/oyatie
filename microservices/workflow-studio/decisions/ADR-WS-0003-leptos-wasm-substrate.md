---
id: ADR-WS-0003
title: Leptos WASM substrate for the workflow-studio canvas rendering tier
microservice: workflow-studio
status: Accepted
date: 2026-05-17
owner: axis-workflow + council-design-system
deciders: council-design-system, council-architecture, axis-workflow, ops-sre-reliability
supersedes: []
superseded_by: []
related: [ADR-0065, ADR-0105, ADR-0131]
related_specs: [/specs/products/workflow-studio.json]
related_artifacts:
  - microservices/workflow-studio/PRD.md (FR-01, FR-02, FR-03, AC-09, AC-12, §"Performance")
  - microservices/workflow-studio/IP-002-visual-canvas-kernel-domain.md
  - microservices/workflow-studio/IP-012-visual-canvas-leptos-wasm-rest-sdk-app.md
purpose: Establish Leptos as the workflow-studio canvas rendering substrate, with an explicit per-µservice ADR justifying the choice for the highest-stakes Leptos surface in oyatie (the n8n-class hero product canvas).
doc_status: published
---

# ADR-WS-0003: Canvas rendering tier — Leptos browser-WASM with signal-driven reactivity

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The workflow-studio visual canvas is the largest Leptos application in oyatie (per PRD §"Purpose" — "the visual canvas is the largest Leptos application in oyatie per ADR-0065"). The canvas must render and remain interactive for graphs up to 5,000 nodes at 60 fps (per `competitor-parity-matrix.md` §"Performance + scale" — "Cold-load 5k-node graph ≤ 3s"); editor TTI cold (CDN-cached) p99 ≤ 2s (PRD §"Performance"); save round-trip p99 ≤ 200ms stable / 100ms GA; collab CRDT merge p99 ≤ 100ms.

ADR-0065 (repo-wide) settled Leptos as oyatie's browser UI substrate for the docs webapp and general application UIs. workflow-studio inherits that decision *de jure*, but workflow-studio's canvas is qualitatively different from a docs site:

- **Render workload**: tens of thousands of canvas primitives (nodes, edges, ports, labels) that change reactively in response to drag gestures and CRDT merges. The docs webapp's workload is mostly static prose.
- **Frame budget**: 16.7ms (60 fps) for drag interactions; a VDOM-diff approach (React-style) cannot sustain this at 5k-node scale without manual virtualization.
- **State density**: editor session state, collab CRDT state, jurisdiction-overlay state, debugger-frame state — all reactive — co-exist in a single shell. Reactive-system efficiency dominates wall-clock performance.
- **WASM/native code sharing**: workflow-studio's `-domain` crates (visual layout algebra, dsl-emitter/loader, CRDT merge engine, overlay renderer) are pure Rust and used unchanged from the browser-WASM target. This is a structural advantage no JS-first stack offers.

Open Question Q2 in PRD §"Open Questions" — "WASM canvas: pure Leptos-Rust-WASM vs Leptos-shell + JS canvas library (reactflow/xyflow)?" with stated bias toward pure Leptos for stable+; JS-canvas only for M03-preview if Leptos canvas not ready. This ADR resolves that question to **pure Leptos** for all milestones, with no JS-canvas fallback.

The decision warrants a workflow-studio-specific ADR (rather than a sole reliance on the repo-wide ADR-0065) because:
1. Workflow-studio is the highest-stakes Leptos surface in oyatie; the cost of a poor substrate choice here is qualitatively larger than for other Leptos surfaces.
2. Per ADR-WS-0001 (CRDT) and ADR-WS-0002 (canonical form), several invariants flow through the canvas (AC-02 round-trip byte-equality, AC-06 never-silent-loss, AC-12 WASM bundle SRI). The canvas substrate must be evaluated against these invariants specifically.
3. The bundle-size, build-toolchain, and SSR-vs-CSR shape choices made here are inherited by every Leptos surface authored against workflow-studio's SDK (post-GA marketplace embeddable canvas).

## Decision

Adopt **Leptos 0.7+** as the workflow-studio canvas rendering substrate, in the following shape:

1. **Compile target**: `wasm32-unknown-unknown` for the browser bundle; `cargo-leptos` orchestrates the build; the canvas runs entirely in the browser (CSR mode for the canvas surface). SSR is enabled for the editor-shell route (login redirect, tenant resolution, OIDC bootstrap) but the canvas itself is CSR — server-rendering a 5k-node graph offers no observable benefit and pays the SSR cost.
2. **Reactivity model**: Leptos signals (`Signal<T>`, `ReadSignal<T>`, `WriteSignal<T>`, `Memo<T>`) for all reactive state. Fine-grained reactivity is the load-bearing performance property: only nodes whose signals changed re-render, regardless of graph size. No `RwSignal` for shared state used across threads (browser is single-threaded; WebWorker boundaries use message-passing, not shared state).
3. **Rendering pipeline**: Leptos's compile-time `view!` macro emits direct DOM operations (no virtual DOM). Node/edge SVG elements are rendered via keyed `<For>` components keyed on `Node::id` and `Edge::id` — guaranteed per ADR-0065 SRI lane and PRD §"Functional Requirements".
4. **Canvas substrate**: SVG-based canvas (not `<canvas>` 2d / WebGL) for accessibility (ARIA roles per node/edge), CSS theming, and DOM-inspector debugging. SVG is the choice of n8n, Camunda, Foundry Pipeline Builder, and Figma's CRDT layer. WebGL is a post-M03 optimization should the 5k-node frame budget fail measurement (kept as a contained `-adapter-webgl` exploration; non-blocking).
5. **No JS canvas library fallback**: explicitly forbid React-Flow, xyflow, JointJS, mxGraph as a fallback. Mixing JS-canvas with the Rust canvas + CRDT layer multiplies the failure modes (two reactive systems, two render trees, two WASM↔JS bridges per frame). The decision is binary.
6. **Bundle splitting**: per-route code splitting via `cargo-leptos`'s WASM bundle profile. Canvas, collab-CRDT, debugger, node-library descriptors each shipped as a separate WASM chunk; each chunk carries an SRI hash per AC-12.
7. **WASM↔JS interop**: minimized via `wasm-bindgen`. JS interop limited to (a) browser APIs not yet bound in `web-sys` (rare; web-sys covers >95% of DOM), (b) third-party WebSocket clients (avoided — using `gloo-net`'s pure-Rust WS bindings), (c) the WASM bootstrap nonce script (single point per PRD §"Security" CSP rule).
8. **Test substrate**: `wasm-bindgen-test` for component-render tests; `playwright-rust` for e2e tests; Lighthouse-style synthetic for TTI budget assertion (PRD §AC-09).
9. **Accessibility**: WCAG 2.2 AA target; ARIA roles per canvas primitive; keyboard navigation matches the Linear-style benchmark cited in `competitor-parity-matrix.md`.

## Alternatives Considered

### Alternative A — React + Vite (TypeScript)

The industry-default browser stack.

- **Pros**
  - Largest hiring pool; largest ecosystem (React-Flow, xyflow, JointJS, dagre).
  - Vite dev server is fast; hot-module-replacement is mature.
  - Most workflow-tool competitors (n8n, Workato, Make) use React + a canvas library.
- **Cons**
  - **VDOM diff cost** scales with tree size; 5k-node graphs require manual virtualization (windowing) to hit 60 fps. The benchmark literature (Marko, Solid, Svelte authors) consistently shows VDOM-based frameworks 5-10x slower than fine-grained reactive frameworks at large list/tree workloads.
  - **No code-sharing** with the Rust-native `-domain` crates. The dsl-emitter, dsl-loader, CRDT merge engine, and overlay renderer would need to be either re-authored in TypeScript or invoked through a WASM↔JS bridge on every signal change. The bridge cost itself dominates 60-fps frame budgets at 5k nodes.
  - Bundle size for React + React-Flow + TS runtime is documented at ~250-400 KB gzip before app code; oyatie's TTI budget cannot absorb that without aggressive code splitting.
  - Forks oyatie's developer-experience surface: every other µservice is Rust, but this one would be TS. Cross-µservice consistency lost.
- **Rejected reason**: code-share with the Rust-native `-domain` crates is load-bearing for AC-02 round-trip byte-equality and AC-06 never-silent-loss invariants. A WASM↔JS round-trip per reactive update is incompatible with the 60-fps frame budget at 5k-node scale.

### Alternative B — SolidJS

Fine-grained reactive framework; closest JS-side analogue to Leptos.

- **Pros**
  - Fine-grained signals match Leptos's reactivity model; 60-fps frame budget achievable.
  - Smaller bundle than React (~12 KB gzip core).
  - Vite-based; fast dev loop.
- **Cons**
  - Same code-share problem as React — no native Rust integration; `-domain` crates would have to run through a WASM↔JS bridge.
  - Smaller ecosystem than React; fewer turn-key canvas libraries (Solid-Flow exists but is preview-grade).
  - Same hiring-pool fragmentation as React (different from oyatie's Rust majority).
- **Rejected reason**: code-share problem remains; Solid's reactivity advantage doesn't outweigh the WASM↔JS bridge cost. Leptos offers Solid-equivalent reactivity *and* native Rust code-share.

### Alternative C — Svelte 5 / SvelteKit

Compile-time reactive framework with runes (Svelte 5).

- **Pros**
  - Compile-time reactivity is fast; small runtime.
  - Svelte 5 runes are a recent, well-designed reactivity model.
- **Cons**
  - Same code-share problem as React/Solid.
  - Smaller ecosystem than React; canvas libraries are scarce.
  - Svelte 5 is recent (2024); production stability is still being demonstrated at large scale.
- **Rejected reason**: code-share problem dominates; Svelte's compile-time advantage doesn't outweigh the boundary cost.

### Alternative D — Yew (Rust browser-WASM)

Yew is React-style Rust-to-WASM with a VDOM.

- **Pros**
  - Native Rust; code-share with `-domain` crates.
  - Familiar React-shaped API for contributors with React background.
- **Cons**
  - VDOM-based; same scale-out problem as React for 5k-node graphs.
  - Smaller ecosystem than Leptos within the Rust browser-WASM space; Leptos has overtaken Yew in commits, contributor count, and download numbers in 2024-2025.
  - SSR support less mature than Leptos; oyatie's editor-shell SSR route would be friction.
- **Rejected reason**: VDOM scaling problem persists; Leptos solves the same code-share goal with better reactivity primitives.

### Alternative E — Dioxus (Rust browser-WASM)

Dioxus is another Rust-to-WASM framework (also targets desktop/mobile).

- **Pros**
  - Native Rust; code-share.
  - Cross-platform story (desktop + mobile + web from one codebase) is appealing for a hypothetical Studio mobile app.
- **Cons**
  - VDOM-based (same scale-out problem).
  - Multi-target ambition spreads engineering effort; Leptos's web-only focus produces better web-specific performance.
  - Smaller browser-WASM-specific install base.
- **Rejected reason**: VDOM + multi-target ambition; the workflow-studio canvas needs a web-specialist substrate.

### Alternative F — Leptos with JS canvas library (React-Flow / xyflow) embedded for canvas surface only

The hybrid suggested in PRD Q2 as the M03-preview fallback.

- **Pros**
  - Could ship faster if pure Leptos canvas is not ready.
  - Inherits the JS-canvas ecosystem's interaction handlers.
- **Cons**
  - Two reactive systems; every signal change crosses WASM↔JS twice per frame.
  - CRDT state and visual state would be in separate runtimes; AC-06 never-silent-loss becomes much harder to verify (state lives in two heaps).
  - CSP and SRI lanes become more complex (JS-canvas chunks need separate SRI hashes; CSP must allow JS-canvas's eval-style internals).
  - "Just for M03 preview" technical-debt patterns historically persist; the carrying cost compounds.
- **Rejected reason**: doubles the failure modes; the AC-02 + AC-06 invariants get qualitatively harder to defend. PRD's explicit anti-pattern stance against fallbacks-that-stick applies.

## Consequences

### Architectural

- The visual-canvas `-adapter-leptos-wasm` crate is the largest in the µservice by line count; it is the canonical example of an `-adapter-leptos-wasm` layer per ADR-0105 Amendment 3.
- The `-domain` crates (visual-canvas-domain, dsl-emitter-domain, dsl-loader-domain, collab-crdt-domain, jurisdiction-overlay-renderer-domain, replay-debugger-frontend-domain) are pure Rust and unit-tested both on native `cargo nextest` and via `wasm-bindgen-test` to verify WASM-target parity.
- The `-app` composition root (`oya-workflow-studio-visual-canvas-app`) emits two binaries: a server-side SSR binary (Axum + Leptos SSR) and the WASM bundle for the browser; `cargo-leptos` orchestrates both.
- WASM bundle SRI hashes (AC-12) are computed during `cargo-leptos` build; the `wasm-bundle-sri` lane verifies them on every PR.
- The CSP per PRD §"Security" (`script-src 'self' 'wasm-unsafe-eval' 'nonce-<random>'`) is the canvas's strict-CSP shape; no JS-canvas inline scripts because the JS-canvas alternative is rejected.

### Downstream impact on other µservices and IPs

1. **IP-002 (visual-canvas kernel + domain)** — Leptos-agnostic; pure Rust. No change.
2. **IP-012 (visual-canvas leptos-wasm + rest + sdk + app)** — Leptos-specific implementation; cargo-leptos build pipeline authored here; SRI lane wired here.
3. **All Studio BCs with `-adapter-leptos-wasm` layer or rendering surface** — adopt the Leptos signals + `view!` macro + keyed `<For>` patterns.
4. **workflow-engine µservice** — unaffected; engine is server-side Axum, not a UI surface.
5. **application µservice** — workflow-studio runs in the application µservice's hosting shell; Leptos SSR routes are mounted there.
6. **observability µservice** — `editor-experience.json` dashboard gains a `wasm_bundle_size_bytes` SLI and per-route `leptos_hydration_time_ms` SLI; release gate checks bundle-size delta.
7. **All future Leptos µservice surfaces** — inherit the Leptos 0.7+ + signals + SVG-canvas + cargo-leptos pattern as the canonical Studio-tier reference; documented in `docs/standards/workflow-studio-canvas.md` (per PHASE-01 in-scope artifacts).
8. **cloud-iac µservice** — CDN edge cache configuration tuned for `.wasm` chunks (Cache-Control: immutable when SRI-hashed; long TTL); per-pack edge cache keys (per PRD §"Security").

### SLOs and CI lanes affected

- `oya-governance-wasm-bundle-sri` — BLOCKER lane on `dev`, `staging` per PHASE-01 (every WASM chunk has SRI; mismatch refuses load).
- `workflow-studio.editor_tti_p99_ms` — Lighthouse synthetic against pack-kr CDN edge; target ≤ 2s p99 GA.
- `workflow-studio.canvas_frame_time_p99_ms` — frame-budget assertion; alarm if p99 > 16.7ms during drag interaction on 5k-node golden graph.
- `workflow-studio.wasm_bundle_size_bytes` — per-chunk + total; release gate alarms on >+50 KB gzip delta.
- `workflow-studio.canvas_hydration_time_p99_ms` — initial hydration after WASM load; bounded by TTI budget.

### Toolchain + supply-chain

- `cargo-leptos` added to the workspace toolchain pin.
- `wasm-bindgen` + `web-sys` + `gloo-net` + `leptos` + `leptos_router` + `leptos_meta` pinned via `Cargo.lock` and reviewed in cargo-deny.
- WASM-bundle SRI computed using SHA-384 per W3C SRI spec; embedded into HTML at SSR time.
- Per-pack CDN cache keys partition the WASM bundles per PRD §"Security" (no cross-tenant cache pollution).

### Performance budget verification (gating GA)

- 5k-node golden graph: cold-load ≤ 3s; drag p99 frame time ≤ 16.7ms; CRDT merge p99 ≤ 100ms — all measured at M03/P01 exit gate per PHASE-01 §"End-to-end drill gates".
- WebGL `-adapter-webgl` exploration: kept under `microservices/workflow-studio/src/crates/oya-workflow-studio-visual-canvas-adapter-webgl/` as a non-blocking exploration; activates only if SVG hits a measured ceiling. Decision deferred to a follow-up ADR if invoked.

### Risk register

- **Risk**: Leptos 0.7+ breaking changes between minor versions. **Mitigation**: pinning + bi-monthly upstream PR review; major-version migrations are explicit ChangeSet IPs.
- **Risk**: SVG render performance ceiling at 5k+ nodes on low-end hardware. **Mitigation**: WebGL adapter exploration in parallel; not on the critical path until measurement establishes need.
- **Risk**: cargo-leptos toolchain instability. **Mitigation**: vendored fallback build script (raw `cargo` + `wasm-bindgen-cli`) authored under IP-012.
- **Risk**: SSR hydration mismatch (server vs client divergence) for editor-shell route. **Mitigation**: hydration-mismatch test in IP-012; alarm on `canvas_hydration_time_p99_ms` anomaly.

## References

- PRD `microservices/workflow-studio/PRD.md` §"Open Questions" Q2, §"Performance", AC-09, AC-12, FR-01, FR-02, FR-03.
- `microservices/workflow-studio/IP-002-visual-canvas-kernel-domain.md`.
- `microservices/workflow-studio/IP-012-visual-canvas-leptos-wasm-rest-sdk-app.md`.
- `microservices/workflow-studio/competitor-parity-matrix.md` §"Performance + scale".
- ADR-0065 — Docs-as-Leptos webapp (inherited; this ADR is the Studio-canvas-specific application of that decision).
- ADR-0105 — 13-layer canonical enum including `adapter-leptos-wasm` (Amendment 3).
- Leptos — `leptos.dev`, `github.com/leptos-rs/leptos`.
- cargo-leptos — `github.com/leptos-rs/cargo-leptos`.
- W3C Subresource Integrity — `www.w3.org/TR/SRI/`.
- WCAG 2.2 — `www.w3.org/TR/WCAG22/`.
- ADR-WS-0001 (CRDT library) — interaction surface with reactive canvas.
- ADR-WS-0002 (DSL canonical form) — projection contract.
