---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: workflow-studio
milestone: M03-studio-preview
phase: P02-native-canvas-shells
impl_plan_id: IP-018-swiftui-canvas-impl
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-frontend
co_owners: [axis-platform-apple]
date: 2026-05-18
related_adrs: [ADR-0185, ADR-0204, ADR-0207]
acceptance_lanes: [a11y-uikit-traits, perf-canvas-60fps, oya-governance-promotion-readiness]
depends_on: [IP-016, IP-022, IP-023]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-018 — SwiftUI canvas impl (Apple native shell)

## Goal

Ship the Apple native (macOS + iPadOS) Workflow Studio shell. Renderer uses SwiftUI `Canvas` for node/edge primitives with `Metal`-backed fast path behind a feature flag for >5k node graphs. Accessibility uses SwiftUI accessibility traits per ADR-0207 + WCAG 2.2 AA. RTL via `environment(\.layoutDirection, .rightToLeft)`. Loro CRDT sync (ADR-0145) via swift-bindings produced from the Rust core. Honors the IP-016 `CanvasAdapter` contract.

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `clients/apple/WorkflowStudio/Package.swift` | create | ~80 LoC; platforms macOS 14, iOS 17; deps grpc-swift, loro-swift, swift-collections |
| `clients/apple/WorkflowStudio/App/WorkflowStudioApp.swift` | create | ~60 LoC; `@main` SwiftUI App, scene root |
| `clients/apple/WorkflowStudio/Canvas/CanvasView.swift` | create | ~320 LoC; SwiftUI `Canvas { ctx, size in … }`; pan/zoom via `GestureState`; LOD ladder |
| `clients/apple/WorkflowStudio/Canvas/CanvasAdapter.swift` | create | ~130 LoC; `protocol CanvasAdapter` impl mirroring IP-016 TS interface |
| `clients/apple/WorkflowStudio/A11y/CanvasAccessibility.swift` | create | ~140 LoC; per-node `accessibilityLabel/Hint/Action`, focus order, live region for drag state |
| `clients/apple/WorkflowStudio/Collab/LoroBinding.swift` | create | ~180 LoC; uses `LoroSwift` (uniffi-swift wrapper around the Rust core IP-022) |
| `clients/apple/WorkflowStudio/Collab/PresenceBinding.swift` | create | ~110 LoC; awareness map (IP-023); overlay cursor rendering |
| `clients/apple/WorkflowStudio/Tests/CanvasIntegrationTests.swift` | create | ~240 LoC; 6 tests |
| `clients/apple/WorkflowStudio/Tests/AccessibilityXCUITests.swift` | create | ~120 LoC; XCUITest assertions on traits + drag-keyboard-alt |
| `microservices/workflow-studio/runbooks/swiftui-canvas-debug.md` | create | ~80 LoC operator playbook |
| `microservices/workflow-studio/decisions/ADR-0185.md` | append §"SwiftUI shell wired" | +6 LoC |

## Code shape

`Canvas/CanvasView.swift` (excerpt):

```swift
struct CanvasView: View {
    @StateObject private var state: CanvasState
    @Environment(\.layoutDirection) private var layoutDirection

    var body: some View {
        Canvas { ctx, size in
            let lod = LodLadder.resolve(state.zoom)
            for node in state.visibleNodes() { renderNode(ctx, node, lod) }
            for edge in state.visibleEdges() { renderEdge(ctx, edge, lod) }
        }
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Workflow canvas")
        .gesture(panGesture.simultaneously(with: zoomGesture))
        .environment(\.layoutDirection, state.tenantRtl ? .rightToLeft : .leftToRight)
    }
}
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `swiftui_canvas_renders_1000_nodes_at_60fps` | Tests/CanvasIntegrationTests.swift | p99 frame ≤ 16.67ms over 30s loop on M-series |
| `swiftui_canvas_keyboard_nav_tab_cycles_nodes` | Tests/AccessibilityXCUITests.swift | XCUIElement focus cycles every node |
| `swiftui_canvas_loro_sync_two_peer_merges` | Tests/CanvasIntegrationTests.swift | Two peers edit; merge equality |
| `swiftui_canvas_presence_shared_cursors` | Tests/CanvasIntegrationTests.swift | Peer cursor renders within 200ms p95 |
| `swiftui_canvas_rtl_environment_flips` | Tests/AccessibilityXCUITests.swift | `layoutDirection == .rightToLeft` flips origin |
| `swiftui_canvas_cross_tenant_isolation` | Tests/CanvasIntegrationTests.swift | Mismatched tenant_id → empty doc |
| `swiftui_canvas_a11y_traits_exposed` | Tests/AccessibilityXCUITests.swift | Every node has label + hint non-empty |
| `swiftui_canvas_drag_keyboard_alt_wcag_2_5_7` | Tests/AccessibilityXCUITests.swift | Keyboard alternative satisfies WCAG 2.5.7 dragging |

Minimum 5 required; 8 specified.

## Evidence to emit

- `evidence/microservices/workflow-studio/swiftui-canvas-perf-{date}.json`
- `evidence/microservices/workflow-studio/swiftui-canvas-a11y-{date}.json` — XCUITest accessibility audit report
- Audit-chain seal: `oya audit-chain seal --kind canvas-perf --ms workflow-studio --shell swiftui --window 30d`
- Metrics: `oya_workflow_studio_canvas_frame_time_ms_bucket{shell="swiftui"}`, `..._loro_merge_latency_ms_bucket{shell="swiftui"}`
- Logs via `OSLog` → OTLP exporter at `clients/apple/WorkflowStudio`.

## Rollback procedure

1. Revert ChangeSet for `clients/apple/WorkflowStudio/`.
2. Remove SwiftPM product from root `Package.swift`.
3. Flip feature flag `workflow_studio_shell_swiftui=disabled`.
4. Pull TestFlight/Mac App Store distribution build.
5. Update PRD §"Supported shells".
6. Emit rollback evidence JSON.

## Blocking dependencies

- IP-016 adapter contract.
- IP-022 Loro Rust core + LoroSwift wrapper.
- IP-023 awareness map.
- ADR-0207 SwiftUI a11y traits mapping table.

## Acceptance gates

```bash
cargo run -p oya-dev-cli -- gate validate a11y-uikit-traits --shell swiftui --pkg clients/apple/WorkflowStudio
cargo run -p oya-dev-cli -- gate validate perf-canvas-60fps --shell swiftui --window 30s
cargo run -p oya-dev-cli -- gate validate oya-governance-promotion-readiness --microservice workflow-studio
xcodebuild test -scheme WorkflowStudioTests
```

## Halt conditions

- Perf lane fails: STOP, file regression IP.
- Any XCUITest accessibility audit violation: STOP.
- Loro merge correctness test fails: STOP, escalate to ADR-0145 owner.

## Exit criteria

1. All 8 tests pass on macOS arm64 + iPadOS simulator CI runners.
2. `a11y-uikit-traits`, `perf-canvas-60fps`, `oya-governance-promotion-readiness` lanes green.
3. Evidence ledger sealed.
4. PRD §"Supported shells" lists SwiftUI shell.
5. Runbook published.
6. ADR-0185 status updated.

## Next IP

[`IP-019-compose-canvas-impl.md`](IP-019-compose-canvas-impl.md)

## References

- ADR-0185 client stack.
- ADR-0204 canvas perf bar.
- ADR-0207 a11y bar (SwiftUI traits table).
- ADR-0145 Loro pin.
- ADR-0208 WebSocket transport.
- Apple Human Interface Guidelines — Accessibility.
- WCAG 2.5.7 Dragging Movements.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/workflow-studio/IP-018-swiftui-canvas-impl.md` matched [`p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/workflow-studio/IP-018-swiftui-canvas-impl.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/ARCHITECTURE.md`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/multi-region.md`, `microservices/workflow-studio/capacity-model.md`].

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-018-swiftui-canvas-impl.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
