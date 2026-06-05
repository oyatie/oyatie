---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: workflow-studio
milestone: M03-studio-preview
phase: P02-native-canvas-shells
impl_plan_id: IP-024-1000-node-perf-bench
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-frontend
co_owners: [axis-perf]
date: 2026-05-18
related_adrs: [ADR-0204]
acceptance_lanes: [perf-canvas-60fps, perf-budget-no-regression, oya-governance-promotion-readiness]
depends_on: [IP-016]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-024 — 1000-node canvas performance benchmark (60fps target)

## Goal

Enforce ADR-0204's perf commitment: 1000+ nodes at 60fps sustained during pan/zoom; p99 frame time ≤ 16.67ms. Build a Playwright-driven bench at `clients/web-sveltekit/tests/canvas-1000-node.bench.ts` that constructs a 1000-node graph, runs a deterministic pan/zoom loop for 30 seconds, captures frame times via `performance.measure()`, and asserts the p99 budget. CI lane `perf-canvas-60fps` consumes the artifact and gates promotion. A second-tier 5000-node bench gated behind the `webgl_escape_hatch=enabled` feature flag verifies the Phase 1.5 ceiling.

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `clients/web-sveltekit/tests/canvas-1000-node.bench.ts` | create | ~220 LoC; Playwright bench; deterministic seeded RNG; pan/zoom loop |
| `clients/web-sveltekit/tests/canvas-5000-node.bench.ts` | create | ~180 LoC; gated on `webgl_escape_hatch` flag |
| `clients/web-sveltekit/tests/fixtures/graph-1000-deterministic.json` | create | ~1000 nodes + 2000 edges fixture (machine-generated) |
| `clients/web-sveltekit/tests/lib/frame-recorder.ts` | create | ~80 LoC; wraps `performance.measure` + emits histogram |
| `clients/web-sveltekit/tests/lib/pan-zoom-script.ts` | create | ~60 LoC; deterministic pan/zoom path |
| `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md` | create | ~120 LoC; bisect playbook + flame-graph capture |
| `crates/oya-dev-cli/src/perf_canvas_60fps_gate.rs` | create | ~140 LoC; gate validator that ingests bench JSON, asserts p99 ≤ 16.67ms |
| `registry/quality/lanes.yaml` | append `perf-canvas-60fps` lane entry | +12 LoC |
| `microservices/workflow-studio/decisions/ADR-0204.md` | append §"Bench landed; p99 budget enforced" | +6 LoC |

## Code shape

`clients/web-sveltekit/tests/canvas-1000-node.bench.ts` (excerpt):

```ts
test('1000-node canvas sustains 60fps p99', async ({ page }) => {
  await page.goto('/studio/bench/1000');
  await page.evaluate(() => window.__loadFixture__('graph-1000-deterministic'));
  const histogram = await page.evaluate(async () => {
    const recorder = new FrameRecorder();
    await runPanZoomScript({ durationMs: 30_000, recorder });
    return recorder.histogram();
  });
  const p99 = quantile(histogram, 0.99);
  expect(p99).toBeLessThanOrEqual(16.67);
  await test.info().attach('frame-histogram.json', { body: JSON.stringify(histogram) });
});
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `canvas_1000_node_p99_under_16_67ms` | canvas-1000-node.bench.ts | p99 frame ≤ 16.67ms over 30s |
| `canvas_1000_node_p95_under_12ms` | canvas-1000-node.bench.ts | p95 frame ≤ 12ms |
| `canvas_1000_node_no_gc_pause_over_50ms` | canvas-1000-node.bench.ts | Long-task GC pause ≤ 50ms across run |
| `canvas_5000_node_p99_under_16_67ms_with_webgl` | canvas-5000-node.bench.ts | gated on flag; p99 ≤ 16.67ms with WebGL escape hatch |
| `canvas_5000_node_perf_floor_without_webgl_documented` | canvas-5000-node.bench.ts | Without WebGL: explicit fail with documented floor (proves the regression detection works) |

Minimum 3 required; 5 specified.

## Evidence to emit

- `evidence/microservices/workflow-studio/canvas-perf-1000-node-{date}.json` — full histogram + p50/p95/p99 + GC pause distribution
- `evidence/microservices/workflow-studio/canvas-perf-5000-node-{date}.json` — WebGL path histogram
- `evidence/microservices/workflow-studio/canvas-perf-bisect-trace-{date}.json` — emitted when bisect runs
- Audit-chain seal: `oya audit-chain seal --kind canvas-perf-bench --ms workflow-studio --window 30d`
- Metrics: `oya_workflow_studio_canvas_bench_frame_ms_bucket{tier=1000|5000}`, `oya_workflow_studio_canvas_bench_gc_pause_ms_bucket`

## Rollback procedure

1. Disable the perf-canvas-60fps lane in `registry/quality/lanes.yaml` (move to `lanes_quarantined` with reason).
2. Halt new promotions of workflow-studio shells until bench restored.
3. Emit rollback evidence JSON with rationale.
4. Revert ChangeSet for bench harness if root cause is in the bench itself.

## Blocking dependencies

- IP-016 — svelte-flow canvas adapter (the bench target).
- ADR-0204 — canvas perf bar.

## Acceptance gates

```bash
buck2 build //:quality-lane-registry-authority-check # lane=perf-canvas-60fps \
  --evidence evidence/microservices/workflow-studio/canvas-perf-1000-node-*.json
buck2 build //:quality-lane-registry-authority-check # lane=perf-budget-no-regression \
  --history evidence/microservices/workflow-studio/canvas-perf-1000-node-*.json \
  --tolerance 5%
buck2 build //:quality-lane-registry-authority-check # lane=oya-governance-promotion-readiness --microservice workflow-studio
pnpm --filter web-sveltekit test:bench
```

## Halt conditions

- p99 > 16.67ms: STOP, perf regression. Run bisect playbook; file regression IP.
- p95 > 12ms: WARN; still allowed but tracked; three consecutive warnings escalate to STOP.
- 5000-node WebGL path fails p99: STOP for 5000-tier; 1000-tier still promotable.

## Exit criteria

1. All 5 bench tests green on CI runners (consistent across 3 consecutive runs).
2. `perf-canvas-60fps` lane added to `registry/quality/lanes.yaml` and active.
3. `perf-budget-no-regression` lane consumes the artifact.
4. Evidence ledger sealed.
5. Runbook published.
6. ADR-0204 status updated.

## Next IP

[`IP-025-codemirror-6-integration.md`](IP-025-codemirror-6-integration.md)

## References

- ADR-0204 canvas perf bar.
- IP-016 svelte-flow integration.
- Chrome DevTools Performance — `https://developer.chrome.com/docs/devtools/performance/`.
- Playwright tracing — `https://playwright.dev/docs/trace-viewer`.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/workflow-studio/IP-024-1000-node-perf-bench.md` matched [`p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/workflow-studio/IP-024-1000-node-perf-bench.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/ARCHITECTURE.md`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/multi-region.md`, `microservices/workflow-studio/capacity-model.md`].

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-024-1000-node-perf-bench.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
