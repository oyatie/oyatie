# IP-sustainability-emission-model — sheets

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: per_query.

Sheets binds the model to cell edit, recalculation, chart render, import/export XLSX, collab cursor sync, CRDT merge, and query-like formula evaluation. The declaration does not claim live FinOps or measured spreadsheet emissions.

## Calibration fixture

- Capacity anchor: `oya/sheets/manifest.json` capacity_model uses per_query scaling, 0.18 baseline CPU, 512 MiB RAM, 40 GiB storage, and Valkey/Postgres/outbound HTTP connections.
- Resource sample: one p50 recalculation query after a cell edit with chart metadata refresh.
- Power coefficients: CPU 0.59, memory 0.0031, storage 0.00090, network 0.057.
- Expected p50: 5.5 mWh and 0.0022g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

The provider_sku_pricing binding is a sheets formula/chart query SKU reference in this IP until cloud-billing has a committed map. No invoice or chargeback side effect is allowed.

## RED fixture contract

Future tests must reject rows that hide recalculation under a generic document request or omit provider/region/cost/watt-hours/CO2 for export XLSX.
