---
doc_class: Tutorial
microservice: sheets
persona: financial-modeller + tenant-developer
date: 2026-05-20
doc_status: published
---

# Tutorial — Build a 100k-cell financial model + collaborate in real-time + export to XLSX

You will: create a multi-sheet financial model, author SUMIF/VLOOKUP/NPV/IRR formulas, invite a collaborator, watch CRDT-merged simultaneous edits, export to XLSX + verify round-trip. Total time ≤ 60 minutes.

## Pre-requisites

- A paid-tier+ sheets cell.
- Two test user accounts (`drill-modeller-a`, `drill-modeller-b`) in tenant `drill-acme`.

## Step 1 — Create a workbook + sheet structure (≤ 5 min)

```sh
oya sheets workbook create \
    --tenant drill-acme \
    --name acme-2026-financial-model \
    --owner drill-modeller-a
```

Add sheets:

```sh
oya sheets sheet add --workbook acme-2026-financial-model --name revenue-forecast
oya sheets sheet add --workbook acme-2026-financial-model --name cost-budget
oya sheets sheet add --workbook acme-2026-financial-model --name cash-flow
oya sheets sheet add --workbook acme-2026-financial-model --name product-sales
oya sheets sheet add --workbook acme-2026-financial-model --name assumptions
```

Each sheet is created within the same workbook + collab session.

## Step 2 — Seed assumptions sheet (≤ 5 min)

In the `assumptions` sheet, author the model constants:

| Cell | Value | Comment |
|---|---|---|
| A1 | "Assumptions" | Header |
| A2 | "Growth rate" | Label |
| B2 | 0.12 | 12 % growth |
| A3 | "Discount rate" | Label |
| B3 | 0.08 | 8 % discount for NPV |
| A4 | "Tax rate" | Label |
| B4 | 0.21 | 21 % federal corporate tax |
| A5 | "Forecast horizon" | Label |
| B5 | 5 | 5-year horizon |

```sh
oya sheets cells bulk-set \
    --workbook acme-2026-financial-model \
    --sheet assumptions \
    --range A1:B5 \
    --values '[["Assumptions",""], ["Growth rate", 0.12], ["Discount rate", 0.08], ["Tax rate", 0.21], ["Forecast horizon", 5]]'
```

Name the ranges:

```sh
oya sheets named-range add --workbook acme-2026-financial-model --name growth_rate --range assumptions!B2
oya sheets named-range add --workbook acme-2026-financial-model --name discount_rate --range assumptions!B3
oya sheets named-range add --workbook acme-2026-financial-model --name tax_rate --range assumptions!B4
oya sheets named-range add --workbook acme-2026-financial-model --name forecast_horizon --range assumptions!B5
```

## Step 3 — Build the product-sales detail sheet (≤ 10 min)

The `product-sales` sheet contains the granular per-product per-month sales data (~ 50 k cells for 4 products × 12 months × 5 years × ~ 50 region/segment combinations).

```sh
oya synthetic seed-sales \
    --workbook acme-2026-financial-model \
    --sheet product-sales \
    --products widgets,gadgets,gizmos,gimcracks \
    --segments enterprise,mid-market,smb \
    --regions us-east,us-west,eu,apac \
    --months 60 \
    --revenue-mean 50000 \
    --revenue-stddev 15000
```

This generates ~ 48 k cells with revenue figures. Verify:

```sh
oya sheets sheet stats --workbook acme-2026-financial-model --sheet product-sales
```

Should show ~ 48 000 cells; ~ 0 formulas; data only.

## Step 4 — Author the revenue-forecast aggregation (≤ 10 min)

In the `revenue-forecast` sheet, build the annual revenue rollups:

| Cell | Formula | Description |
|---|---|---|
| A1 | "Revenue Forecast" | Header |
| A3 | "Year" | Header |
| A4 | "Widgets" | Product label |
| A5 | "Gadgets" | |
| A6 | "Gizmos" | |
| A7 | "Gimcracks" | |
| A8 | "TOTAL" | |
| B3 | 2026 | Year header |
| B4 | `=SUMIF('product-sales'!A:A, "widgets", 'product-sales'!E:E)` | Sum widgets sales for 2026 |
| B5 | `=SUMIF('product-sales'!A:A, "gadgets", 'product-sales'!E:E)` | |
| B6 | `=SUMIF('product-sales'!A:A, "gizmos", 'product-sales'!E:E)` | |
| B7 | `=SUMIF('product-sales'!A:A, "gimcracks", 'product-sales'!E:E)` | |
| B8 | `=SUM(B4:B7)` | Total revenue |
| C3..F3 | 2027..2030 | Future year headers |
| C4..F7 | `=B4 * (1 + growth_rate)^(C3-B3)` etc. | Project growth |
| C8..F8 | `=SUM(C4:C7)` etc. | Yearly totals |
| B10 | `=NPV(discount_rate, B8:F8)` | Net Present Value |
| B11 | `=IRR(B8:F8)` | Internal Rate of Return |

```sh
oya sheets cells bulk-set \
    --workbook acme-2026-financial-model \
    --sheet revenue-forecast \
    --range A1:F11 \
    --values @./revenue-forecast-formulas.json
```

Watch the recalc cascade in the Grafana `sheets-recalc-duration` panel; expected ~ 200-400 ms for the 48-cell formula tree.

## Step 5 — Invite a collaborator + edit concurrently (≤ 10 min)

```sh
oya sheets share \
    --workbook acme-2026-financial-model \
    --grant-to drill-modeller-b \
    --role editor
```

The Cedar gate `sheets::share::grant` evaluates; allowed if `drill-modeller-a` is owner-or-tenant-admin.

In a second terminal session (as `drill-modeller-b`), open the workbook + edit cell `B4`:

```sh
# Terminal 1 (drill-modeller-a) — concurrently
oya sheets cell edit --workbook acme-2026-financial-model --sheet revenue-forecast --address C4 --value "=B4 * 1.15"

# Terminal 2 (drill-modeller-b) — concurrently
oya sheets cell edit --workbook acme-2026-financial-model --sheet revenue-forecast --address B5 --value "=SUMIF('product-sales'!A:A, \"gadgets\", 'product-sales'!E:E) * 1.02"
```

Watch the Loro CRDT sync panel (`sheets-crdt-sync-lag`); both edits should propagate to the other user's view within ~ 150 ms.

Now provoke a conflict:

```sh
# Both terminals — simultaneously edit B6
oya sheets cell edit --workbook acme-2026-financial-model --sheet revenue-forecast --address B6 --value "=SUMIF('product-sales'!A:A, \"gizmos\", 'product-sales'!E:E)"
```

Loro's CRDT detects the simultaneous-edit-conflict; the UI surfaces a conflict marker on cell B6; both edits preserved; user picks. The audit chain emits `crdt_conflict_resolved`.

## Step 6 — Export to XLSX + verify round-trip (≤ 10 min)

```sh
oya sheets workbook export \
    --workbook acme-2026-financial-model \
    --format xlsx \
    --output ./acme-2026-financial-model.xlsx \
    --fidelity-level best-effort
```

Open the export in Microsoft Excel or LibreOffice Calc. Verify:

- All 6 sheets are present (revenue-forecast, cost-budget, cash-flow, product-sales, assumptions, [implied workbook overview if added]).
- Formulas evaluate identically (NPV, IRR, SUMIF results match).
- Named ranges preserved.
- Number formats preserved.

Now re-import the XLSX:

```sh
oya sheets workbook import \
    --tenant drill-acme \
    --file ./acme-2026-financial-model.xlsx \
    --workbook-name acme-2026-financial-model-round-trip
```

Run the fidelity check:

```sh
oya sheets xlsx-fidelity-check \
    --workbook-original acme-2026-financial-model \
    --workbook-round-trip acme-2026-financial-model-round-trip
```

Expected: 100 % formula fidelity, ~ 98 % formatting fidelity. The 2 % gap is itemised.

## Step 7 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant drill-acme --since 1h --workbook acme-2026-financial-model
```

Expected events:

- `workbook_created`
- `sheet_added` × 5
- `cell_set` × N (one per bulk-set range; not per cell)
- `named_range_added` × 4
- `share_granted` × 1
- `crdt_conflict_resolved` × 1
- `workbook_exported` × 1
- `workbook_imported` × 1
- `xlsx_fidelity_check_completed` × 1

## What you've learned

- The multi-sheet workbook structure + cross-sheet formula reference.
- The 400+ formula library (SUMIF, VLOOKUP, NPV, IRR, SUM, named ranges).
- The Loro CRDT real-time collaboration model.
- The XLSX round-trip fidelity check.
- The audit-chain shape for sheets operations.

Next tutorial: `tutorials/build-pivot-tables-and-charts.md` — author pivot tables + multiple chart types on the financial model.
