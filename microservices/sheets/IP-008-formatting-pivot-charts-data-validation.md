---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-sheets-preview
phase: P01-sheets-foundation
impl_plan_id: IP-008-formatting-pivot-charts-data-validation
status: pending
owner: axis-sheets + council-design-system
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1]
depends_on: [IP-003, IP-004]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: formatting + pivot-tables + charts + data-validation — kernel + domain + usecase + api + adapter (custom Leptos canvas chart renderer)

## Intent

Author the four cross-cutting BCs that compose with cell-grid:
- `formatting`: number/date/currency/percent/custom formats; conditional formatting rule engine.
- `pivot-tables`: pivot configuration + aggregation evaluator (sum/count/avg/min/max/stddev/median/distinct_count).
- `charts`: bar/line/pie/scatter/area/combo/sparkline; custom Leptos canvas renderer (no JS chart-lib dependency).
- `data-validation`: dropdown/range/custom-formula/number-range/date-range/text-length rules.

## ChangeSet boundary

~25 crates across the four BCs.

## Code Shape

`charts-adapter-leptos-wasm/src/render.rs` (excerpt):

```rust
use leptos::*;
use web_sys::CanvasRenderingContext2d;

#[component]
pub fn ChartCanvas(chart: Chart) -> impl IntoView {
    view! {
        <canvas
            id={format!("chart-{}", chart.chart_id)}
            on:mount=move |_| {
                let ctx = get_canvas_2d(&chart.chart_id);
                match chart.chart_type {
                    ChartType::Bar => render_bar(&ctx, &chart),
                    ChartType::Line => render_line(&ctx, &chart),
                    // ...
                }
            }
            data-data-class="BEHAVIORAL_TENANT_PRODUCT"
        />
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-sheets-formatting-{kernel,domain} -p oya-sheets-pivot-tables-{kernel,domain} \
  -p oya-sheets-charts-{kernel,domain,adapter-leptos-wasm} -p oya-sheets-data-validation-{kernel,domain}
cargo nextest run -p oya-sheets-formatting-domain
cargo nextest run -p oya-sheets-pivot-tables-domain
cargo nextest run -p oya-sheets-charts-domain
cargo nextest run -p oya-sheets-data-validation-domain
cargo build --target wasm32-unknown-unknown -p oya-sheets-charts-adapter-leptos-wasm
```

## Test Plan

Per Phase-01 thresholds; chart-render benchmark suite asserts AC-13 budget.

## Halt Conditions

- Chart render p95 > 200ms on benchmark corpus — STOP.

## Next IP

[`IP-009-import-export-xlsx-calamine-rust-xlsxwriter-sandboxed.md`](IP-009-import-export-xlsx-calamine-rust-xlsxwriter-sandboxed.md)

## References

- PRD FR-07 + FR-08 + FR-09 + FR-10 + AC-13.
- ADR-0065 (Leptos for browser UI).
