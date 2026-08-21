---
doc_class: Runbook
title: Chart render degraded (custom Leptos canvas budget breach)
microservice: sheets
severity: "Sev-3"
status: Accepted
owner_team: axis-sheets + council-design-system
date: 2026-05-17
related_artifacts:
  - app/sheets/failure-modes.md (FM-07)
  - app/sheets/threat-model.md §"T-D-09"
  - app/sheets/PRD.md FR-09 + AC-13
doc_status: published
---

# Runbook: Chart render degraded

## Purpose

Sheets renders charts (bar/line/pie/scatter/area/combo/sparkline) via a custom Leptos canvas renderer per PRD §FR-09. Per-chart render budget is ≤ 200ms p95 (AC-13). Excess chart count on a single dashboard sheet OR custom renderer regression breaches the budget. This runbook covers detection and mitigation.

## Trigger

ONE of:

1. **`oya_sheets_chart_render_seconds{quantile="0.95"} > 0.2` for ≥ 10 min** (AC-13 budget breach).
2. **`oya_sheets_chart_count_per_sheet > 100` for an active sheet** (soft cap).
3. **Tenant reports**: "charts on this dashboard render slowly".
4. **Browser-side beacon**: chart render p99 from real-user-monitoring exceeds budget.

## Severity

- Single-tenant single-sheet: Sev-3.
- Cluster-wide chart-renderer regression: Sev-2.

## Impact

- Charts render slowly or partially; viewport feels janky.
- Tenant trust impact (especially dashboard-class tenants).

## Pre-checks

1. Identify slow sheets: query `oya_sheets_chart_render_slow_top_n`.
2. Verify chart count per sheet: `oya_sheets_chart_count_per_sheet_top_n`.
3. Verify Leptos canvas renderer health: WASM bundle SRI hash matches expected.
4. Identify whether failure is per-(tenant, sheet) (lots of charts) or cluster-wide (renderer regression).

## Recovery Path A — Chart count cap exceeded (single sheet)

| Step | Action |
|---|---|
| 1 | Verify per-sheet chart count > 100. |
| 2 | Tenant-facing banner: "this sheet has 100+ charts; rendering may be slow. Consider splitting into multiple sheets". |
| 3 | Lazy-render activated: charts outside viewport scheduled-for-distinct-tracked-work until scrolled into view. |
| 4 | gtm-customer-success may proactively reach out: "we noticed your sheet has X charts; would design help reduce?". |

## Recovery Path B — Custom Leptos canvas renderer regression

| Step | Action |
|---|---|
| 1 | Run chart-render benchmark corpus: `cargo run -p oya-sheets-charts-adapter-leptos-wasm --bin bench`. |
| 2 | If corpus fails: roll back Sheets release per `runbooks/formula-engine-rollback.md` pattern. |
| 3 | Cluster-wide tenant notification. |

## Recovery Path C — Chart series too complex (single chart breach)

Cause: tenant authored a chart with millions of data points (e.g., scatter plot of 1M cells).

| Step | Action |
|---|---|
| 1 | Per-chart data-point cap (default 100k); tenant-facing banner: "chart has too many data points; consider aggregating or sampling". |
| 2 | Sampled-render activated: chart renders 100k sampled points; banner shown. |
| 3 | gtm-customer-success may reach out. |

## Recovery Path D — Browser-side performance issue (real-user-monitoring)

Cause: tenant's browser is underpowered; rendering slow client-side despite server-side budget met.

| Step | Action |
|---|---|
| 1 | Verify server-side chart-render-prep budget within target. |
| 2 | If server-side OK: tenant-side issue; no server action. |
| 3 | gtm-customer-success may suggest a different browser / hardware. |

## Verification

After recovery:
- `oya_sheets_chart_render_seconds{quantile="0.95"} ≤ 0.2`.
- No `oya_sheets_chart_count_per_sheet > 100` active warnings.
- Tenant-side synthetic chart render test passes.

## Post-incident updates

- Postmortem if Sev-2 (cluster-wide).
- If repeated chart-count-cap hits: surface to council-design-system for product UX guidance ("you should be using a dashboard µservice for 100+ charts").
- If renderer regression: regression test corpus expansion.

## References

- `app/sheets/PRD.md` FR-09 + AC-13.
- `app/sheets/threat-model.md` T-D-09.
- `app/sheets/failure-modes.md` FM-07.
- Leptos canvas — `leptos.dev/docs`.
- Real-user-monitoring metrics — `web.dev/vitals/`.
