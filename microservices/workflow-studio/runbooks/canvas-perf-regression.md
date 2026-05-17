---
doc_class: Runbook
title: Canvas performance regression (TTI / render / save budget)
microservice: workflow-studio
severity: "Sev-3 (single-budget breach) / Sev-2 (multi-budget OR tenant-impacting)"
status: Accepted
owner_team: axis-workflow + council-design-system + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/workflow-studio/PRD.md §"Performance" + §"Acceptance Criteria" AC-09, AC-10
  - microservices/workflow-studio/failure-modes.md (FM-06 perf budget breach)
  - microservices/workflow-studio/threat-model.md §"T-D-04" LLM latency cascade
  - /specs/products/workflow-studio.json §goals.performance
doc_status: published
---

# Runbook: Canvas performance regression

## Trigger

ONE of:

1. **Editor TTI p99 > 2s for ≥ 15 min** (GA budget per `/specs/products/workflow-studio.json` §goals.performance).
2. **Canvas render p95 > 16ms (60fps) for ≥ 15 min**.
3. **Save round-trip p99 > 200ms (stable) OR > 100ms (GA) for ≥ 15 min**.
4. **Cold-load of 5k-node graph > 3s** for any tenant.
5. **WASM bundle download time p99 > 1s** (CDN issue).
6. **`oya_workflow_studio_perf_budget_breached_total{budget=<...>}` rate > 0**.

## Severity

- Single budget breached, single pack: Sev-3.
- Multi-budget OR tenant-impacting (active tenants > 10) OR > 1h persistent: Sev-2.
- Tenant unable to author OR cross-pack: Sev-1.

## Impact

- Tenant authoring slowed (editor TTI > 2s); per FR-01..FR-09 the canvas is the load-bearing UX.
- Burn-rate budget consumed (per `cost-budget.md`); MTTR target ≤ 60s for auto-rollback.
- Risk: tenant abandonment at perceived slowness; AC-09/10 are competitive-parity gates with n8n + Workato.

## Pre-checks

1. Identify breached budget: `Grafana dashboards/editor-experience.json` panel "TTI p99 vs budget" — which budget? which pack?
2. Identify when the regression started: `dashboards/editor-experience.json` long-range view → mark the deploy boundary.
3. Identify the WASM bundle version served: `curl -sI https://studio-<pack>.oyatie.dev/canvas.wasm | grep -i 'studio-version'`.
4. Verify CDN reach: `dashboards/editor-experience.json` panel "CDN edge cache hit rate" — if cache hit < 95%, CDN drift.
5. Identify p99 contributor: profiling-sampler `dashboards/editor-experience.json` panel "FCP / LCP / TTI breakdown".

## Recovery Path A — CDN cache drift (stale chunk served)

Cause: post-deploy CDN purge did not propagate; stale WASM bundle served from a subset of edges; tenants on those edges see old + sometimes incompatible code.

| Step | Action | Time |
|---|---|---|
| 1 | Verify CDN purge propagation: `curl -sI https://studio-<pack>-cdn-edge-<id>.oyatie.dev/canvas.wasm | grep ETag` from each edge. | ≤ 5 min |
| 2 | If subset drift: re-issue purge: `cargo run -p oya-dev-cli -- cdn purge --pack <pack> --path '/v*/canvas.wasm'`. | ≤ 1 min |
| 3 | Verify propagation complete in ≤ 60s p99 (per `threat-model.md` T-D-03 SLI). | ≤ 5 min |
| 4 | If propagation fails: fall through to Path B (full CDN failover). | – |

## Recovery Path B — CDN edge outage (full CDN failover)

Cause: CDN provider edge region unreachable; tenants in pack served fallback origin.

| Step | Action |
|---|---|
| 1 | Verify CDN provider status page. |
| 2 | Activate fallback origin-direct serving (bypass CDN): `kubectl -n workflow-studio patch ingress studio-cdn -p '{"spec":{"rules":[{"host":"studio-<pack>.oyatie.dev","http":{"paths":[{"backend":{"service":{"name":"studio-origin"}}}]}}]}}'`. |
| 3 | Tenants now hit origin directly; p99 will be 1.5-2x CDN baseline; acceptable for outage window. |
| 4 | Re-enable CDN when provider returns; verify TTI returns to budget. |

## Recovery Path C — Deploy regression (newest WASM bundle slower)

Cause: a recent release introduced a perf regression (e.g., un-keyed list render, new Leptos signal storm, heavier dep).

| Step | Action |
|---|---|
| 1 | Confirm via deploy-boundary correlation in Grafana. |
| 2 | Engage `runbooks/rollback.md` (rollback-by-pointer pattern): `cargo run -p oya-dev-cli -- vcs rollback --microservice workflow-studio --env <env> --to-sha <prior-sha> --reason "TTI perf regression"`. |
| 3 | Verify TTI returns to budget within ≤ 15 min. |
| 4 | File a regression issue against the deployed change; assignee root-causes (Chrome DevTools MCP + Leptos profiling). |
| 5 | Add a CI lane regression test: `tests/load/tti-budget.js` updated to assert against the specific path that regressed. |

## Recovery Path D — Bundle size growth

Cause: dependency upgrade OR new feature crate added > 200KB to the WASM bundle; cold-load slowed.

| Step | Action |
|---|---|
| 1 | Verify bundle size in CI lane: `oya-governance-wasm-bundle-size --microservice workflow-studio --max-size 5MB`. |
| 2 | If exceeds budget: trace which crate added size via `cargo bloat --release --target wasm32-unknown-unknown`. |
| 3 | Mitigate: lazy-load the heavy feature crate via dynamic import; redeploy. |
| 4 | If feature is core: file an ADR ("accept higher TTI for this feature" OR "redesign for budget"). |

## Recovery Path E — Render thrash (60fps budget breach during interaction)

Cause: canvas re-renders >16ms p95 during drag/zoom; Leptos signal storm OR un-keyed list render.

| Step | Action |
|---|---|
| 1 | Reproduce in local Chrome DevTools (record performance trace during drag of 100-node graph). |
| 2 | Identify hot-path: typically `<NodeList />` re-render storm OR `<EdgePath />` recomputing all edges. |
| 3 | Patch: add `key=` attributes on lists; memoize edge paths; redeploy. |
| 4 | Verify p95 returns to 16ms budget; record evidence at `evidence/perf/canvas-render-<change_id>.json`. |

## Recovery Path F — Save-roundtrip regression

Cause: save p99 > 200ms; usually engine spec-store backpressure OR Postgres editor-session lock contention.

| Step | Action |
|---|---|
| 1 | Identify upstream cost: `dashboards/editor-experience.json` panel "Save round-trip breakdown" (Studio→Engine + Engine→Postgres + Studio→Postgres). |
| 2 | If Engine spec-store slow: engage `microservices/workflow-engine/runbooks/spec-store-perf.md`. |
| 3 | If Studio→Postgres slow: see `runbooks/session-storm-throttle.md` for Postgres-side guidance. |
| 4 | If transient (1 deploy worth): wait + re-measure. If sustained: rollback per Path C. |

## Verification

After recovery:
- `oya_workflow_studio_editor_tti_seconds{quantile="0.99"} < 2.0` for ≥ 30 min.
- `oya_workflow_studio_canvas_render_seconds{quantile="0.95"} < 0.016` for ≥ 30 min.
- `oya_workflow_studio_save_round_trip_seconds{quantile="0.99"} < 0.2` (stable) OR `< 0.1` (GA).
- No active alerts on Studio self-SLI.
- Synthetic Lighthouse-style budget run: pass.

## Post-incident updates

- Postmortem within 5 business days.
- If CDN drift recurring: harden purge propagation SLI (`runbooks/canvas-perf-regression.md` Path A).
- If deploy regressions recurring: tighten the `oya-foundry-fitness-perf-budget` lane to BLOCKER (already is, but assert it actually fires).
- If bundle-growth recurring: institute monthly bundle-size review per `docs/standards/agentic-dev-team-optimization.md`.

## References

- `microservices/workflow-studio/PRD.md` AC-09, AC-10.
- `microservices/workflow-studio/failure-modes.md` FM-06.
- `microservices/workflow-studio/threat-model.md` T-D-03 (CDN purge gap), T-D-04 (LLM cascade).
- `/specs/products/workflow-studio.json` §metrics + §goals.performance.
- Google SRE Workbook ch. 5 (multi-window burn rate).
- Web.dev Core Web Vitals — `web.dev/vitals/`.
- Lighthouse perf budgets — `web.dev/performance-budgets-101/`.
- Leptos perf docs — `book.leptos.dev/performance/index.html`.
