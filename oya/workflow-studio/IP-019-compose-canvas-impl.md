---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: workflow-studio
milestone: M03-studio-preview
phase: P02-native-canvas-shells
impl_plan_id: IP-019-compose-canvas-impl
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-frontend
co_owners: [axis-platform-android]
date: 2026-05-18
related_adrs: [ADR-0185, ADR-0204, ADR-0207]
acceptance_lanes: [a11y-talkback-conformance, perf-canvas-60fps, oya-governance-promotion-readiness]
depends_on: [IP-016, IP-022, IP-023]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-019 — Compose canvas impl (Android native shell)

## Goal

Ship the Android Compose Workflow Studio shell. Renderer uses Jetpack Compose `Canvas` modifier with `drawIntoCanvas` for node/edge primitives and `RenderEffect` shaders for LOD downsampling. Accessibility via `Modifier.semantics` per ADR-0207. RTL via `CompositionLocalProvider(LocalLayoutDirection provides LayoutDirection.Rtl)`. Loro CRDT sync (ADR-0145) via Kotlin bindings produced by `uniffi-kotlin` from the Rust core (IP-022). Honors the IP-016 `CanvasAdapter` contract.

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `clients/android/workflowstudio/build.gradle.kts` | create | ~80 LoC; targetSdk 34, Compose BOM, grpc-kotlin, loro-android |
| `clients/android/workflowstudio/src/main/kotlin/com/oya/studio/WorkflowStudioApp.kt` | create | ~60 LoC; Application + Activity bootstrap |
| `clients/android/workflowstudio/src/main/kotlin/com/oya/studio/canvas/CanvasComposable.kt` | create | ~320 LoC; `@Composable fun WorkflowCanvas` with pan/zoom modifier + LOD ladder |
| `clients/android/workflowstudio/src/main/kotlin/com/oya/studio/canvas/CanvasAdapter.kt` | create | ~130 LoC; impl `CanvasAdapter` matching IP-016 contract |
| `clients/android/workflowstudio/src/main/kotlin/com/oya/studio/a11y/CanvasSemantics.kt` | create | ~140 LoC; per-node `Modifier.semantics { contentDescription = …; role = Role.Button }`; live region for drag state |
| `clients/android/workflowstudio/src/main/kotlin/com/oya/studio/collab/LoroBinding.kt` | create | ~180 LoC; uses `loro-kotlin` (uniffi) wrapper around Rust core |
| `clients/android/workflowstudio/src/main/kotlin/com/oya/studio/collab/PresenceBinding.kt` | create | ~110 LoC; awareness map IP-023; overlay cursor draw |
| `clients/android/workflowstudio/src/androidTest/kotlin/com/oya/studio/canvas/CanvasIntegrationTest.kt` | create | ~240 LoC; 6 tests |
| `clients/android/workflowstudio/src/androidTest/kotlin/com/oya/studio/a11y/AccessibilityScannerTest.kt` | create | ~120 LoC; Espresso + Android Accessibility Scanner integration |
| `microservices/workflow-studio/runbooks/compose-canvas-debug.md` | create | ~80 LoC operator playbook |
| `microservices/workflow-studio/decisions/ADR-0185.md` | append §"Compose shell wired" | +6 LoC |

## Code shape

`CanvasComposable.kt` (excerpt):

```kotlin
@Composable
fun WorkflowCanvas(state: CanvasState, modifier: Modifier = Modifier) {
    val lod by remember { derivedStateOf { LodLadder.resolve(state.zoom) } }
    Canvas(
        modifier = modifier
            .fillMaxSize()
            .pointerInput(Unit) { detectTransformGestures { _, pan, zoom, _ ->
                state.applyPanZoom(pan, zoom)
            } }
            .semantics {
                contentDescription = "Workflow canvas"
                liveRegion = LiveRegionMode.Polite
            }
    ) {
        state.visibleNodes().forEach { renderNode(it, lod) }
        state.visibleEdges().forEach { renderEdge(it, lod) }
    }
}
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `compose_canvas_renders_1000_nodes_at_60fps` | CanvasIntegrationTest.kt | p99 frame ≤ 16.67ms over 30s; via `FrameMetricsAggregator` |
| `compose_canvas_keyboard_nav_tab_cycles_nodes` | CanvasIntegrationTest.kt | TalkBack focus order traverses all nodes |
| `compose_canvas_loro_sync_two_peer_merges` | CanvasIntegrationTest.kt | Two peers edit; merge equality |
| `compose_canvas_presence_shared_cursors` | CanvasIntegrationTest.kt | Peer cursor renders within 200ms p95 |
| `compose_canvas_rtl_layout_direction_flips` | CanvasIntegrationTest.kt | `LayoutDirection.Rtl` flips origin |
| `compose_canvas_cross_tenant_isolation` | CanvasIntegrationTest.kt | Mismatched tenant_id → empty doc |
| `compose_canvas_accessibility_scanner_green` | AccessibilityScannerTest.kt | Scanner emits zero errors |
| `compose_canvas_drag_keyboard_alt_wcag_2_5_7` | AccessibilityScannerTest.kt | Keyboard alt satisfies WCAG 2.5.7 |

Minimum 5 required; 8 specified.

## Evidence to emit

- `evidence/microservices/workflow-studio/compose-canvas-perf-{date}.json` — FrameMetricsAggregator histogram
- `evidence/microservices/workflow-studio/compose-canvas-a11y-{date}.json` — Scanner JSON output
- Audit-chain seal: `oya audit-chain seal --kind canvas-perf --ms workflow-studio --shell compose --window 30d`
- Metrics: `oya_workflow_studio_canvas_frame_time_ms_bucket{shell="compose"}`
- Logs via Logcat → OTLP exporter.

## Rollback procedure

1. Revert ChangeSet for `clients/android/workflowstudio/`.
2. Remove module from `clients/settings.gradle.kts`.
3. Flip feature flag `workflow_studio_shell_compose=disabled`.
4. Pull APK/AAB from Play Internal track.
5. Update PRD §"Supported shells".
6. Emit rollback evidence JSON.

## Blocking dependencies

- IP-016 adapter contract.
- IP-022 Loro Rust core + uniffi-kotlin wrapper.
- IP-023 awareness map.
- ADR-0207 Compose semantics mapping table.

## Acceptance gates

```bash
cargo run -p oya-dev-cli -- gate validate a11y-talkback-conformance --shell compose --module clients/android/workflowstudio
cargo run -p oya-dev-cli -- gate validate perf-canvas-60fps --shell compose --window 30s
cargo run -p oya-dev-cli -- gate validate oya-governance-promotion-readiness --microservice workflow-studio
./gradlew :clients:android:workflowstudio:connectedAndroidTest
```

## Halt conditions

- Perf lane fails: STOP.
- TalkBack conformance / Accessibility Scanner emits any error: STOP.
- Loro merge correctness test fails: STOP, escalate to ADR-0145 owner.

## Exit criteria

1. 8 tests pass on Android 13 + 14 emulator CI matrix.
2. `a11y-talkback-conformance`, `perf-canvas-60fps`, `oya-governance-promotion-readiness` lanes green.
3. Evidence ledger sealed.
4. PRD updated.
5. Runbook published.
6. ADR-0185 status updated.

## Next IP

[`IP-020-gtk-drawingarea-impl.md`](IP-020-gtk-drawingarea-impl.md)

## References

- ADR-0185 client stack.
- ADR-0204 canvas perf bar.
- ADR-0207 a11y bar (Compose semantics table).
- ADR-0145 Loro pin.
- ADR-0208 WebSocket transport.
- Material 3 Compose accessibility guidance.
- WCAG 2.5.7 Dragging Movements.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/workflow-studio/IP-019-compose-canvas-impl.md` matched [`p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/workflow-studio/IP-019-compose-canvas-impl.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/ARCHITECTURE.md`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/multi-region.md`, `microservices/workflow-studio/capacity-model.md`].

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-019-compose-canvas-impl.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
