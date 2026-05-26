# Migration plan: Leptos prototype → SolidJS app-shell

**Authority:** ADR-0372 (accepted 2026-05-26)
**Milestone:** M-APP-SHELL-FRONTEND
**Status:** Slice 1 scaffolded — migration in progress

---

## What this document covers

This plan maps every surface in
`crates/oya-application-shell-frontend-prototype` to its SolidJS destination,
defines the retirement sequence, and records what stays Rust (the backend).
The Leptos prototype is **not modified or deleted this round** — it remains the
reference until each surface is verified in the SolidJS shell.

---

## Surface inventory and migration map

### Leptos prototype surfaces (from app.rs)

| Leptos component / construct | SolidJS destination | Slice | Status |
|---|---|---|---|
| `App` (root mount) | `src/app.tsx` + `src/routes/index.tsx` | 1 | **Done** |
| `ShellRail` (aside nav) | `src/components/ShellRail.tsx` | 1 | **Done** |
| `ShellHeader` (sticky header, route strip, command trigger) | `src/components/ShellHeader.tsx` | 1 | **Done** |
| `HeroPanel` (title, FD-001 close strip, render-arch strip) | Inlined in `src/routes/index.tsx` | 1 | **Done** |
| `DashboardIsland` (context switcher + all panels below) | `src/components/DashboardIsland.tsx` | 1 | **Done** |
| `MetricCard` | `MetricCardView` in DashboardIsland | 1 | **Done** |
| `ModuleCard` | `ModuleCardView` in DashboardIsland | 1 | **Done** |
| `WorkItem` / `ScheduleItem` / `MessageItem` | `WorkItemView` / `ScheduleItemView` / `MessageItemView` | 1 | **Done** |
| `ApprovalItem` | `ApprovalItemView` in DashboardIsland | 1 | **Done** |
| `WorkflowNode` canvas (linear display) | `WorkflowCanvas` + `WorkflowNodeView` | 1 | **Done — linear; full canvas in slice 3** |
| Node inspector panel | `workflow-node-inspector` in DashboardIsland | 1 | **Done** |
| `IntelligenceSuggestion` copilot rail | `IntelligenceSuggestionView` | 1 | **Done** |
| `OntologyFact` object graph | `OntologyFactView` | 1 | **Done** |
| `UtilityPanels` (notifications + settings drawers) | `src/components/UtilityPanels.tsx` | 2 | Pending |
| `SidePeek` (object quick-view aside) | `src/components/SidePeek.tsx` | 2 | Pending |
| `BusinessLogicRow` table (BUSINESS_LOGIC_ROWS const) | `src/components/BusinessLogicTable.tsx` | 2 | Pending |
| `WorkflowTool` toolbar (Select / Connect / Simulate) | Part of Workflow Studio slice | 3 | Pending |
| Workflow Studio canvas (full drag + connect) | Rust→WASM compute module per ADR-0372 D3 | 3 | Pending |
| `ProductSurface` tab switcher (Work Hub: Workflow/Messenger/Mail/Community) | `src/components/WorkHub.tsx` | 2 | Pending |
| `LocalDraft` + `HubItem` (local draft state) | SolidJS `createSignal` + `createStore` | 2 | Pending |
| Command palette (`data-command-trigger`) | `src/components/CommandPalette.tsx` | 2 | Pending |
| `render_envelope.rs` types | `src/lib/render-envelope.ts` | 1 | **Done** |
| `prototype-interactions.js` chrome mounts | Replaced by SolidJS signal-driven components | 1–2 | Partial (slice 1 covers ShellRail + ShellHeader) |

### Surfaces in `render_envelope.rs` data model

All Rust structs are ported to `src/lib/render-envelope.ts` (TypeScript
interfaces, one-to-one). The Rust SSR endpoint
`GET /api/render-envelope/:context` remains the backend source of truth;
the SolidJS shell fetches it via `fetchRenderEnvelope()` in `src/lib/api.ts`.

---

## Retirement sequence

### Slice 1 (this PR — scaffold)
- Scaffold `microservices/app-shell-frontend/` per ADR-0131 flat µservice layout.
- Port: App root, ShellRail, ShellHeader, HeroPanel, DashboardIsland (all
  panels, metric grid, workflow canvas linear view, intelligence, ontology).
- Port: `render_envelope.rs` types → TypeScript.
- Wire OpenAPI codegen pipeline (`pnpm codegen`) for `ops-workspace-shell-v1`
  and `hr-api` contracts.
- **Leptos prototype: untouched.**

### Slice 2 (next PR)
- Port: UtilityPanels (notifications + settings drawers), SidePeek, BusinessLogicTable,
  WorkHub surface tabs, LocalDraft state, CommandPalette.
- Replace remaining `prototype-interactions.js` chrome logic with SolidJS signals.
- Add SolidStart SSR route for `/api/render-envelope/:context` proxy (bridges
  the Leptos backend during transition, then points at the real Rust service).
- **Leptos prototype: still untouched.**

### Slice 3 (compute-WASM slice)
- Implement Workflow Studio full drag-connect canvas as a Rust→WASM compute module
  per ADR-0372 D3 (wasm-bindgen, mounted into the SolidJS shell as a web component).
- This is the one place the WASM compile target is justified: the canvas is
  compute-bound + DOM-light (SVG/canvas, not DOM-heavy grid).
- Wire `@wasm-bindgen/...` JS glue; expose via `src/components/WorkflowCanvas.wasm.tsx`.

### Slice 4 (retirement gate)
- TTI / Lighthouse budget check in CI must pass vs Leptos baseline (ADR-0372 D1).
- `oya gate validate contract-client-codegen` must be green (ADR-0372 D2).
- WASM compute module passes widget tests inside the SolidJS shell (ADR-0372 D3).
- Mark `crates/oya-application-shell-frontend-prototype` as archived in its
  `Cargo.toml` description and remove from the primary build target list.
- Remove the Leptos crate from production build paths (CI gate added).
- Update `specs/platform-architecture.json` frontend stack entry.

---

## What stays Rust (backend — out of scope for this migration)

| Component | Stays Rust | Reason |
|---|---|---|
| HTTP router kernel (`oya-http-router-kernel`) | Yes | Backend kernel per ADR-0372 §4 |
| Hyper adapter (`oya-http-runtime-hyper-adapter`) | Yes | Backend runtime |
| `render_envelope.rs` — server-side SSR logic | Yes | Backend data derivation; SolidJS fetches the JSON output |
| `server_mock_catalog.rs` | Yes | Dev-mode catalog; replaced by real service in production |
| OpenAPI 3.2.0 contracts (`.yaml` files) | Yes (source of truth) | SolidJS generates typed clients from them; Rust owns authorship |
| All microservices (tenancy, cloud-iac, hr, …) | Yes | Backend unchanged per ADR-0372 §4 |
| Wasmtime server-side sandbox (ADR-0023) | Yes | Server-side WASM, unrelated to frontend |
| Workflow Studio canvas WASM module (slice 3) | Rust→WASM | Compute-bound; mounted into SolidJS shell, not a Leptos component |

---

## OpenAPI codegen pipeline (ADR-0372 D2)

**Tool:** `openapi-typescript@7.8.0` (MIT). Chosen over `orval` because it
produces type-only `.d.ts` output (no runtime code), which is lower surface
area, and it supports OpenAPI 3.2.0 natively.

**Contracts consumed:**

| Contract file | Generated output | OpenAPI version |
|---|---|---|
| `contracts/ops-workspace-shell-v1.openapi.yaml` | `generated/ops-workspace-shell.d.ts` | 3.2.0 |
| `microservices/hr/contracts/openapi-v1.yaml` | `generated/hr-api.d.ts` | 3.2.0 |

**CI enforcement:** `pnpm codegen:check` (`scripts/codegen-check.mjs`) exits
non-zero if generated files are missing or older than their source contracts.
Wire into CI as a required check alongside `pnpm typecheck`.

**Commands:**
```sh
# Generate typed clients from OpenAPI contracts
pnpm codegen

# Verify generated files are up-to-date (CI gate)
pnpm codegen:check

# TypeScript compile check (no emit)
pnpm typecheck
```

---

## License gate (ADR-0372 D5, ADR-0013)

All frontend dependencies are OSI-clean:

| Package | License | Version |
|---|---|---|
| solid-js | MIT | 1.9.7 |
| @solidjs/router | MIT | 0.14.10 |
| @solidjs/start | MIT | 1.1.0 |
| vite | MIT | 6.3.5 |
| vite-plugin-solid | MIT | 2.11.6 |
| openapi-typescript | MIT | 7.8.0 |
| typescript | Apache-2.0 | 5.8.3 |
| @types/node | MIT | 22.15.21 |

No GPL, LGPL, AGPL, or proprietary dependencies.
