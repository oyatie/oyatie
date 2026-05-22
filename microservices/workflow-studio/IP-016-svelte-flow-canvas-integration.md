---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: workflow-studio
milestone: M03-studio-preview
phase: P02-native-canvas-shells
impl_plan_id: IP-016-svelte-flow-canvas-integration
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-frontend
co_owners: [axis-a11y]
date: 2026-05-18
related_adrs: [ADR-0185, ADR-0204, ADR-0207]
acceptance_lanes: [perf-canvas-60fps, a11y-axe-zero-violations, oya-vcs-promotion-readiness]
depends_on: []
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-016 — svelte-flow canvas integration (Phase 1 SvelteKit)

## Goal

Wire `@xyflow/svelte` (svelte-flow) into the Phase 1 SvelteKit Workflow Studio at `microservices/workflow-studio/clients/web-sveltekit/lib/canvas/`. Use the adapter pattern (per ADR-0204) so a future Phase 2 swap to `oya-canvas-leptos` (IP-017) is bounded blast radius. Honors ADR-0207 a11y bar (WCAG 2.2 AA, keyboard nav, screen-reader live region, RTL support). This IP is the originating canonical adapter contract that every native shell (IP-018/IP-019/IP-020/IP-021) implements.

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `clients/web-sveltekit/package.json` | edit | pin `@xyflow/svelte` (current 1.x line, verified 2026-05-18); add devDeps `@axe-core/playwright`, `playwright` |
| `clients/web-sveltekit/lib/canvas/CanvasAdapter.ts` | create | ~180 LoC; adapter interface (canonical contract); shared types |
| `clients/web-sveltekit/lib/canvas/SvelteFlowCanvasAdapter.ts` | create | ~260 LoC; svelte-flow-backed impl |
| `clients/web-sveltekit/lib/canvas/types.ts` | create | ~80 LoC; `Node`, `Edge`, `Viewport` shared model (matches `contracts/workflow-graph.openapi.yaml`) |
| `clients/web-sveltekit/lib/canvas/lod.ts` | create | ~80 LoC; LOD rendering rules (full / simplified / rect-only) |
| `clients/web-sveltekit/lib/canvas/a11y/keyboard-nav.ts` | create | ~140 LoC; WCAG 2.5.7 drag-keyboard-alt; arrow keys move nodes |
| `clients/web-sveltekit/lib/canvas/a11y/live-region.ts` | create | ~80 LoC; ARIA live region for drag state |
| `clients/web-sveltekit/lib/canvas/rtl.ts` | create | ~40 LoC; reads tenant locale; applies `dir="rtl"` to root |
| `clients/web-sveltekit/contracts/workflow-graph.openapi.yaml` | create or update | full; canonical Node/Edge schema |
| `clients/web-sveltekit/tests/canvas-integration.spec.ts` | create | ~300 LoC; 8 tests |
| `clients/web-sveltekit/tests/canvas-a11y.spec.ts` | create | ~180 LoC; 4 tests |
| `microservices/workflow-studio/runbooks/svelte-flow-canvas-debug.md` | create | ~100 LoC playbook |
| `microservices/workflow-studio/decisions/ADR-0204.md` | append §"Phase 1 svelte-flow shipped" | +6 LoC |

## Architecture sketch

```
clients/web-sveltekit/
  lib/
    canvas/
      CanvasAdapter.ts          // adapter interface (canonical contract)
      SvelteFlowCanvasAdapter.ts // svelte-flow-backed impl
      types.ts                  // shared Node / Edge / Viewport model
      lod.ts                    // LOD rendering rules
      rtl.ts                    // tenant-locale RTL switch
      a11y/
        keyboard-nav.ts         // WCAG 2.5.7 drag-keyboard-alt
        live-region.ts          // live region for drag state
    collab/
      loro-binding.ts           // Loro CRDT sync (IP-022)
      presence-binding.ts       // Loro awareness (IP-023)
    editor/
      cm6/                      // CodeMirror 6 (IP-025)
```

## Code shape

`CanvasAdapter.ts` (excerpt):

```ts
export interface CanvasAdapter {
  mount(parent: HTMLElement, opts: AdapterOpts): void;
  setNodes(nodes: ReadonlyArray<Node>): void;
  setEdges(edges: ReadonlyArray<Edge>): void;
  setViewport(viewport: Viewport): void;
  onSelect(handler: (selected: ReadonlyArray<NodeId>) => void): Disposable;
  onConnect(handler: (from: NodeId, to: NodeId) => void): Disposable;
  destroy(): void;
}
```

`SvelteFlowCanvasAdapter.ts` (excerpt):

```ts
export class SvelteFlowCanvasAdapter implements CanvasAdapter {
  private app: SvelteComponent;
  mount(parent: HTMLElement, opts: AdapterOpts) {
    this.app = mount(SvelteFlowRoot, { target: parent, props: {
      onlyRenderVisibleElements: true,
      nodesFocusable: true,
      dir: opts.rtl ? 'rtl' : 'ltr',
      ...
    } });
    installKeyboardNav(parent);
    installLiveRegion(parent);
  }
}
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `svelte_flow_renders_1000_nodes_at_60fps_smoke` | canvas-integration.spec.ts | 1000-node smoke runs without crashes; passes IP-024 bench |
| `svelte_flow_node_drag_emits_position_change` | canvas-integration.spec.ts | Drag fires position change event with correct delta |
| `svelte_flow_edge_connect_emits_on_connect` | canvas-integration.spec.ts | Drag from source handle to target handle fires `onConnect` |
| `svelte_flow_only_render_visible_elements_active` | canvas-integration.spec.ts | Off-screen nodes do not render in DOM |
| `svelte_flow_lod_ladder_at_zoom_levels` | canvas-integration.spec.ts | Zoom < 0.25 → rect-only DOM; 0.25-0.75 → simplified |
| `svelte_flow_rtl_dir_propagates_to_root` | canvas-integration.spec.ts | `dir="rtl"` on root flips origin |
| `svelte_flow_destroy_unmounts_cleanly` | canvas-integration.spec.ts | After `destroy()`, no DOM nodes remain |
| `svelte_flow_adapter_contract_methods_present` | canvas-integration.spec.ts | All `CanvasAdapter` methods exist + correct signatures |
| `svelte_flow_axe_core_zero_violations` | canvas-a11y.spec.ts | axe-core scan returns 0 violations |
| `svelte_flow_keyboard_nav_tab_cycles_nodes` | canvas-a11y.spec.ts | Tab/Shift-Tab cycles every node |
| `svelte_flow_drag_keyboard_alt_wcag_2_5_7` | canvas-a11y.spec.ts | Arrow keys move selected node satisfies WCAG 2.5.7 |
| `svelte_flow_live_region_announces_drag_state` | canvas-a11y.spec.ts | ARIA live region emits "Moved node X to (px, py)" |

Minimum 8 required; 12 specified.

## Evidence to emit

- `evidence/microservices/workflow-studio/svelte-flow-correctness-{date}.json`
- `evidence/microservices/workflow-studio/svelte-flow-a11y-axe-{date}.json`
- Audit-chain seal: `oya audit-chain seal --kind canvas-phase1 --ms workflow-studio --window 30d`
- Metrics: `oya_workflow_studio_canvas_visible_nodes`, `oya_workflow_studio_canvas_dom_nodes`, `oya_workflow_studio_canvas_event_latency_ms_bucket{event=...}`

## Risk + mitigation

- **Risk:** svelte-flow performance ceiling at 5k+ nodes. **Mitigation:** WebGL escape hatch documented in Phase 1.5; Phase 2 `oya-canvas-leptos` (IP-017) substrate scaffolded.
- **Risk:** svelte-flow upstream BC break. **Mitigation:** adapter interface + pinned lock-file; per-version conformance test in `canvas-integration.spec.ts`.
- **Risk:** WCAG 2.5.7 not natively supported in svelte-flow. **Mitigation:** custom keyboard-nav layer in `a11y/keyboard-nav.ts`.

## Rollback procedure

1. Revert ChangeSet for `clients/web-sveltekit/lib/canvas/`.
2. Flip feature flag `workflow_studio_canvas=disabled` → Studio enters read-only viewer mode (no edit affordance; banner displayed).
3. Unpin `@xyflow/svelte` from `package.json`.
4. Emit rollback evidence JSON.

## Blocking dependencies

- ADR-0185, ADR-0204, ADR-0207 — must be merged.
- `contracts/workflow-graph.openapi.yaml` — Node/Edge schema canonical.

## Acceptance gates

```bash
cargo run -p oya-dev-cli -- gate validate perf-canvas-60fps \
  --evidence evidence/microservices/workflow-studio/svelte-flow-correctness-*.json
cargo run -p oya-dev-cli -- gate validate a11y-axe-zero-violations --target svelte-flow
cargo run -p oya-dev-cli -- gate validate oya-vcs-promotion-readiness --microservice workflow-studio
pnpm --filter web-sveltekit test:integration canvas
```

## Halt conditions

- p99 > 16.67ms: STOP, perf regression.
- axe-core violation: STOP.
- Adapter contract test fails: STOP — contract drift blocks every native shell IP.

## Exit criteria

1. All 12 tests green on CI.
2. `perf-canvas-60fps`, `a11y-axe-zero-violations`, `oya-vcs-promotion-readiness` lanes green.
3. Evidence ledger sealed.
4. Runbook published.
5. ADR-0204 Phase 1 section updated.
6. Adapter contract published as canonical; downstream IPs (018/019/020/021) reference it.

## Next IP

[`IP-017-leptos-canvas-scaffold.md`](IP-017-leptos-canvas-scaffold.md)

## References

- ADR-0185 client stack matrix.
- ADR-0204 canvas perf bar + Phase 1 path.
- ADR-0207 a11y bar.
- ADR-0064 canonical base + localization overlay.
- svelte-flow upstream — `https://svelteflow.dev/`.
- axe-core — `https://github.com/dequelabs/axe-core`.
- WCAG 2.5.7 Dragging Movements.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/workflow-studio/IP-016-svelte-flow-canvas-integration.md` matched [`p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/workflow-studio/IP-016-svelte-flow-canvas-integration.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/ARCHITECTURE.md`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/multi-region.md`, `microservices/workflow-studio/capacity-model.md`].

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-016-svelte-flow-canvas-integration.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
