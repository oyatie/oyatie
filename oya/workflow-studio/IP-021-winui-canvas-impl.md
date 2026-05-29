---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: workflow-studio
milestone: M03-studio-preview
phase: P02-native-canvas-shells
impl_plan_id: IP-021-winui-canvas-impl
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-frontend
co_owners: [axis-platform-windows]
date: 2026-05-18
related_adrs: [ADR-0185, ADR-0204, ADR-0207]
acceptance_lanes: [a11y-uia-conformance, perf-canvas-60fps, oya-governance-promotion-readiness]
depends_on: [IP-016, IP-022, IP-023]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-021 — WinUI 3 canvas impl (Windows native shell)

## Goal

Ship the WinUI 3 Windows shell of Workflow Studio. Renderer uses `Microsoft.Graphics.Canvas` (Win2D) on top of a XAML `CanvasControl` with composition-layer fast paths for pan/zoom. Accessibility uses UIA (UI Automation) per ADR-0207, RTL via `FlowDirection="RightToLeft"`, and Loro CRDT sync (ADR-0145) via the shared .NET binding (cross-compiled from the canonical Rust core through `cbindgen`/`uniffi-cs`). Honors the IP-016 `CanvasAdapter` contract end-to-end.

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `clients/winui3/WorkflowStudio/WorkflowStudio.csproj` | create | ~80 LoC (TargetFramework=net8.0-windows10.0.19041.0, WinUI 3 + Win2D + grpc-dotnet refs) |
| `clients/winui3/WorkflowStudio/App.xaml` + `App.xaml.cs` | create | ~60 + ~80 LoC bootstrap, theme dictionary |
| `clients/winui3/WorkflowStudio/Canvas/CanvasControl.xaml` | create | ~50 LoC; `CanvasControl` (Win2D) + overlay UIA peer |
| `clients/winui3/WorkflowStudio/Canvas/CanvasControl.xaml.cs` | create | ~340 LoC; viewport, LOD ladder, pan/zoom handlers, draw via `args.DrawingSession` |
| `clients/winui3/WorkflowStudio/Canvas/CanvasAdapter.cs` | create | ~130 LoC; implements adapter contract (mirrors IP-016 TS interface) |
| `clients/winui3/WorkflowStudio/A11y/CanvasAutomationPeer.cs` | create | ~160 LoC; exposes `IRawElementProviderFragmentRoot` + per-node `IRawElementProviderSimple` children |
| `clients/winui3/WorkflowStudio/A11y/LiveRegion.cs` | create | ~50 LoC; UIA `LiveSetting=Polite` peer for drag state per ADR-0207 |
| `clients/winui3/WorkflowStudio/Collab/LoroBinding.cs` | create | ~180 LoC; uses Rust core via `uniffi-cs` bindings (IP-022) |
| `clients/winui3/WorkflowStudio/Collab/PresenceBinding.cs` | create | ~120 LoC; awareness from IP-023; renders shared cursors via overlay layer |
| `clients/winui3/WorkflowStudio/Tests/CanvasIntegrationTests.cs` | create | ~240 LoC; 6 tests (see below) |
| `clients/winui3/WorkflowStudio/Tests/UiaConformanceTests.cs` | create | ~120 LoC; Accessibility Insights for Windows CLI driver |
| `microservices/workflow-studio/runbooks/winui3-canvas-debug.md` | create | ~80 LoC operator playbook |
| `microservices/workflow-studio/decisions/ADR-0185.md` | append §"WinUI 3 shell wired" | +6 LoC |

## Code shape

`Canvas/CanvasControl.xaml.cs` (excerpt):

```csharp
public sealed partial class CanvasControl : UserControl, ICanvasAdapter {
    private readonly CanvasState _state;
    private readonly LoroBinding _loro;

    public CanvasControl() {
        this.InitializeComponent();
        _loro = LoroBinding.Connect(WorkflowDocumentId.Current);
        _state = new CanvasState(_loro);
        CanvasSurface.Draw += OnDraw;
        AttachGestures();
    }

    private void OnDraw(CanvasControl sender, CanvasDrawEventArgs args) {
        var ds = args.DrawingSession;
        var lod = LodLadder.Resolve(_state.Zoom);
        foreach (var node in _state.VisibleNodes()) RenderNode(ds, node, lod);
        foreach (var edge in _state.VisibleEdges()) RenderEdge(ds, edge, lod);
    }

    protected override AutomationPeer OnCreateAutomationPeer()
        => new CanvasAutomationPeer(this, _state);
}
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `WinUI_Canvas_Renders_1000Nodes_At_60fps` | Tests/CanvasIntegrationTests.cs | p99 frame ≤ 16.67ms across 30s loop |
| `WinUI_Canvas_KeyboardNav_Tab_Cycles_Nodes` | Tests/CanvasIntegrationTests.cs | Tab/Shift-Tab moves focus through nodes; PageUp/Down jumps regions |
| `WinUI_Canvas_LoroSync_TwoPeer_Merges` | Tests/CanvasIntegrationTests.cs | Two peers edit; CRDT merges; node set equal |
| `WinUI_Canvas_Presence_SharedCursors` | Tests/CanvasIntegrationTests.cs | Peer cursor renders within 200ms p95 |
| `WinUI_Canvas_FlowDirection_RTL` | Tests/CanvasIntegrationTests.cs | `FlowDirection="RightToLeft"` flips origin; UIA exposes `IsRTL=true` |
| `WinUI_Canvas_CrossTenantIsolation` | Tests/CanvasIntegrationTests.cs | Mismatched tenant_id room key → empty doc |
| `WinUI_Canvas_UIA_TreePattern_Exposes_Nodes` | Tests/UiaConformanceTests.cs | UIA TreeWalker finds every node; AutomationPeer.GetName non-empty |
| `WinUI_Canvas_AccessibilityInsights_NoCriticalIssues` | Tests/UiaConformanceTests.cs | AI for Windows CLI emits 0 critical or serious issues |

Minimum 5 required; 8 specified.

## Evidence to emit

- `evidence/microservices/workflow-studio/winui3-canvas-perf-{date}.json` — frame-time histogram
- `evidence/microservices/workflow-studio/winui3-canvas-a11y-{date}.json` — Accessibility Insights for Windows JSON output
- Audit-chain seal: `oya audit-chain seal --kind canvas-perf --ms workflow-studio --shell winui3 --window 30d`
- Metrics: `oya_workflow_studio_canvas_frame_time_ms_bucket{shell="winui3"}`, `oya_workflow_studio_canvas_loro_merge_latency_ms_bucket{shell="winui3"}`
- Structured logs target=`clients/winui3/WorkflowStudio` via Serilog → OTLP exporter.

## Rollback procedure

1. Revert the `clients/winui3/WorkflowStudio/` directory ChangeSet.
2. Remove the project from `clients.sln`.
3. Flip feature flag `workflow_studio_shell_winui3=disabled`.
4. Pull MSIX artifacts from `dist/windows/winui3/` distribution channel.
5. Update PRD §"Supported shells" to remove the WinUI 3 row.
6. Emit rollback evidence `evidence/microservices/workflow-studio/winui3-rollback-{date}.json`.

## Blocking dependencies

- IP-016 — adapter contract.
- IP-022 — Loro Rust core + uniffi-cs bindings.
- IP-023 — awareness map shape.
- ADR-0207 — UIA mapping table.

## Acceptance gates

```bash
cargo run -p oya-dev-cli -- gate validate a11y-uia-conformance --shell winui3 --proj clients/winui3/WorkflowStudio
cargo run -p oya-dev-cli -- gate validate perf-canvas-60fps --shell winui3 --window 30s
cargo run -p oya-dev-cli -- gate validate oya-governance-promotion-readiness --microservice workflow-studio
dotnet test clients/winui3/WorkflowStudio/Tests
```

## Halt conditions

- `perf-canvas-60fps` lane fails: STOP, file regression IP.
- Any UIA conformance violation marked Critical/Serious: STOP, file a11y-defect IP.
- Loro merge correctness test fails: STOP, escalate to ADR-0145 owner.

## Exit criteria

1. 8 tests pass on Windows 10 + Windows 11 CI runners (x64 + arm64).
2. `a11y-uia-conformance`, `perf-canvas-60fps`, `oya-governance-promotion-readiness` lanes green.
3. Evidence ledger sealed.
4. PRD updated.
5. Runbook published.
6. ADR-0185 status reflects WinUI 3 wired.

## Next IP

[`IP-022-loro-crdt-sync-binding.md`](IP-022-loro-crdt-sync-binding.md)

## References

- ADR-0185 client stack matrix.
- ADR-0204 canvas perf bar.
- ADR-0207 accessibility bar (UIA mapping).
- ADR-0145 Loro CRDT pin.
- ADR-0208 WebSocket transport.
- Win2D docs — `https://microsoft.github.io/Win2D/`.
- UIA spec — `https://learn.microsoft.com/windows/win32/winauto/`.
- Accessibility Insights for Windows — `https://accessibilityinsights.io/docs/windows/overview/`.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/workflow-studio/IP-021-winui-canvas-impl.md` matched [`p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/workflow-studio/IP-021-winui-canvas-impl.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/ARCHITECTURE.md`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/multi-region.md`, `microservices/workflow-studio/capacity-model.md`].

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-021-winui-canvas-impl.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
