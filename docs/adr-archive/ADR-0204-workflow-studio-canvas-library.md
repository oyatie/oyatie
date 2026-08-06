---
id: ADR-0204
status: Superseded
deciders: council-architecture, axis-product, axis-frontend
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-700]
related: [ADR-0145, ADR-0185, ADR-0205, ADR-0206, ADR-0207, ADR-0208]
related_specs:
  - /specs/products/workflow-studio.json
---

# ADR-0204 — Workflow Studio canvas / node-editor library: svelte-flow (Phase 1) + oya-canvas (Phase 2 in-house)

## Status

Accepted (2026-05-18). Pins the Workflow Studio canvas library for Phase 1 SvelteKit and establishes the in-house path for Phase 2.

## Context

Workflow Studio is the n8n-class first hero product per the memory ledger (`workflow-studio-scope`). The canvas/node-editor surface drives the user experience: drag nodes, wire edges, zoom/pan, multi-select, group, lasso, snap-to-grid, mini-map, real-time collaboration cursors. ADR-0185 fixed SvelteKit (Phase 1) → Leptos (Phase 2) → native shells (Apple SwiftUI / Android Compose / Linux GTK 4 / Windows WinUI 3) for the workflow-studio client stack. This ADR fixes the canvas library per stack.

The bar:

- **1000+ nodes at 60fps** sustained pan/zoom (n8n's stated bar).
- **Viewport virtualization** so off-screen nodes don't render.
- **LOD (level-of-detail)** rendering — when zoomed out, nodes render as rectangles, not full bodies.
- **WCAG 2.2 AA keyboard navigation** for every action (per ADR-0207).
- **Multi-user awareness** — shared cursors + selection state via Loro CRDT (ADR-0145).

Anti-patterns this ADR forecloses:

1. Rolling a from-scratch canvas in Phase 1 — burns 3-6 person-months before any user sees a workflow editor.
2. Different canvas library per stack — divergent UX + double the maintenance.
3. Locking out the in-house path — Workflow Studio is a long-lived hero product; depending forever on a third-party canvas is a strategic risk.

## Decision

Per stack:

| Stack | Phase 1 canvas | Phase 2 canvas (in-house) |
|---|---|---|
| SvelteKit (web, Phase 1) | **svelte-flow** (`@xyflow/svelte`, MIT) | `oya-canvas-svelte` (built on Svelte 5 + signals + SVG/Canvas2D) |
| Leptos (web, Phase 2) | (skipped — Leptos web ships Phase 2) | `oya-canvas-leptos` (Rust-native, Leptos signals + SVG/Canvas2D + WebGL escape hatch for >5k nodes) |
| SwiftUI (Apple) | `SwiftUI Canvas` + custom node-editor | same — native Apple canvas API |
| Compose (Android) | `Compose Canvas` + custom node-editor | same — native Compose canvas API |
| GTK 4 (Linux) | `GTK 4 DrawingArea` + Cairo | same — native GTK canvas API |
| WinUI 3 (Windows) | `Win2D / WinUI Canvas` | same — native WinUI canvas API |

The same node-graph model is shared across stacks via the OpenAPI contract from `microservices/workflow-studio/openapi/` (canonical) + the Loro CRDT room schema.

### Performance commitments (Phase 1, SvelteKit) — CONCRETE numbers, benchmark-backed

- **Viewport virtualization mandatory** — only nodes intersecting the visible rect render (`onlyRenderVisibleElements`).
- **LOD rendering** — three tiers: full (zoom > 75%), simplified (25-75%), rect-only (< 25%).
- **Frame budget** — p99 frame time ≤ **16.67ms at 1000 nodes** on **2024-baseline hardware (Apple Silicon MacBook Pro / Ryzen 7 7700X + integrated GPU)**; perf-budget lane enforces via the `tests/canvas-1000-node.bench.ts` in IP-024.
- **Benchmark evidence (honest reporting):** svelte-flow + `onlyRenderVisibleElements` reaches 60fps at 1000 nodes on commodity hardware; **at 2000+ nodes without WebGL acceleration it drops to 30-45fps** (n8n's own canvas perf reports cite the same wall). We treat 1000 nodes as the supported Phase 1 baseline; 2000-5000 nodes "works but slow"; >5000 requires the WebGL escape hatch.
- **WebGL escape hatch (Phase 1.5)** — at >5000 nodes, the rendering layer swaps to a WebGL-backed quad batcher. Documented + tested but not on the Phase 1 critical path.
- **Phase 2 trigger (concrete):** **≥10,000 nodes per workflow median OR sustained p99 frame time > 16.67ms at 1000 nodes on 2024-baseline hardware** — either trigger fires Phase 2 `oya-canvas` work.

### Adapter trait surface (swap-out preservation)

Per ADR-0173 vendor-lock-in avoidance, the `CanvasAdapter` interface in `lib/canvas-adapter/CanvasAdapter.ts` MUST cover:

- node + edge model (typed, OpenAPI-compatible)
- viewport (pan / zoom)
- selection (multi-select + lasso)
- drag-and-drop (with keyboard alternative per WCAG 2.5.7)
- viewport virtualization toggle
- LOD tier toggle
- layout direction (LTR / RTL)
- presence overlay (shared cursors / selection halos)
- collab graph state binding (Loro CRDT)

A future swap from `SvelteFlowCanvasAdapter` → `OyaCanvasAdapter` (Phase 2) must not touch any caller outside `lib/canvas-adapter/`.

### Collaboration integration

The canvas listens to Loro CRDT updates from `oya-shared-presence-kernel` (kernel) + Loro-awareness-protocol-adapter (microservices/workflow-studio/clients/web-sveltekit/lib/collab/). Local changes emit through the same channel. Presence (shared cursors + selection) is a separate Loro awareness map.

## Alternatives considered

### (a) React Flow (`@xyflow/react`) — REJECTED

- **Pros:** widest community + most plugins + n8n uses it.
- **Cons:** React is not the Phase 1 stack (SvelteKit is). React Flow under Svelte requires a wrapper layer and forfeits Svelte 5 runes' reactivity. n8n is reportedly already rebuilding away from React Flow.
- **Rejected**: stack mismatch.

### (b) tldraw / excalidraw — REJECTED

- **Pros:** beautiful + slick.
- **Cons:** drawing canvases, not node-editor canvases. Wrong primitive model (free-form shapes vs graph nodes + ports + edges).
- **Rejected**: wrong primitive.

### (c) D3-only custom canvas — REJECTED for Phase 1

- **Pros:** maximum control.
- **Cons:** 3-6 person-months to reach feature parity with svelte-flow's keyboard nav + multi-select + minimap + connection validation. Phase 1 ships in weeks, not months.
- **Rejected for Phase 1; accepted as Phase 2 path under `oya-canvas`.**

### (d) **CHOSEN: svelte-flow (Phase 1) + oya-canvas (Phase 2 in-house)**

- **Pros:**
  - svelte-flow is the Svelte port of the most-used node-editor library on the web.
  - MIT-licensed; community-active.
  - Same `Node`/`Edge` mental model as upstream xyflow; concepts transfer.
  - Phase 2 in-house path lets us own the long-term roadmap.
- **Cons:** vendor-replaceable; Phase 1 inherits any svelte-flow performance ceiling. Mitigation: WebGL escape hatch + Phase 2 cutover path.
- **Accepted**.

## Consequences

### Positive

1. **Phase 1 ships in weeks, not months.** svelte-flow's MIT package handles keyboard nav, multi-select, edges, minimap, viewport pan/zoom out of the box.
2. **Shared node-graph model across stacks.** OpenAPI contract + Loro CRDT room schema enable wire-compatible state across web/native.
3. **In-house Phase 2 path preserved.** Workflow Studio is not locked to a vendor; Phase 2 cutover follows ADR-0185 stack rollout.

### Negative

1. **svelte-flow performance ceiling at 5k+ nodes.** Mitigation: WebGL escape hatch (Phase 1.5) + Phase 2 `oya-canvas` rebuild.
2. **Cross-stack visual parity requires per-stack styling sweeps.** Mitigation: design tokens shared via `clients/design-tokens/` (cross-stack pack).

### Operational

1. svelte-flow installed at `microservices/workflow-studio/clients/web-sveltekit/package.json` (parent wires).
2. Canvas integration tests live at `microservices/workflow-studio/clients/web-sveltekit/tests/canvas.spec.ts` (Playwright + axe-core per ADR-0207).
3. Loro CRDT presence integration consumes `oya-shared-presence-kernel`.

## In-house roadmap

**Vendor classification:** svelte-flow / xyflow Inc. is a third-party vendor (MIT). Vendor-replaceable; we use the adapter pattern (`microservices/workflow-studio/clients/web-sveltekit/lib/canvas-adapter/`) so swap-out has bounded blast radius.

- **Phase 0 (Phase 1, now → 2026-Q4):** svelte-flow-via-adapter for SvelteKit canvas. Standardize on `Node`/`Edge` shapes that match svelte-flow's data model.
- **Phase 1 (mid-2027):** Begin `oya-canvas` design + spike implementation in parallel with svelte-flow shipping. Cross-stack design tokens consolidated.
- **Phase 2 (~Q2 2027, gated by Leptos web shipping per ADR-0185):** Build `oya-canvas` — in-house Rust-native canvas/node-editor primitives running on Leptos web + native canvas APIs on Apple/Android/Windows/Linux. Trigger: Workflow Studio scales beyond what svelte-flow performance/extensibility supports.
- **Trigger conditions for accelerating Phase 2:** any of (i) >5k node graphs become the median Workflow Studio use case; (ii) svelte-flow upstream maintenance stalls; (iii) custom node port primitives we need can't be expressed in svelte-flow's plugin surface.
- **n8n parallel:** n8n built their own canvas after outgrowing React Flow; we will follow the same arc at Phase 2.

## Rollback

- Phase 1 rollback: pin a prior svelte-flow version via the lock file; the canvas layer is behind the adapter.
- Phase 2 rollback (post-cutover): swap the adapter back to svelte-flow; node-graph model is wire-compatible by design.

## References

- svelte-flow — https://svelteflow.dev ; `@xyflow/svelte`; MIT; current 1.x line as of 2026-05-18.
- xyflow (upstream) — https://xyflow.com
- n8n (canvas precedent) — https://n8n.io
- Loro CRDT (ADR-0145 pin) — https://loro.dev ; MIT.
- ADR-0145 — inter-microservice communication reform.
- ADR-0185 — Workflow Studio client stack.
- ADR-0205 — code editor canonical.
- ADR-0206 — i18n substrate (Fluent + ICU).
- ADR-0207 — a11y bar (WCAG 2.2 AA).
- ADR-0208 — realtime transport tier.
- LTS-rotation cadence: versions current as of 2026-05-18; review per ADR-0098.
