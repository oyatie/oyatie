# IP-sustainability-emission-model — notes

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: per_request.

Notes uses request-scoped accounting for note create/open, full-text search, graph render, AI assist, tag/link suggestion, and vault organization. The declaration preserves the advisory boundary and does not claim measured tenant carbon output.

## Calibration fixture

- Capacity anchor: `oya/notes/manifest.json` capacity_model uses per_user scaling, 0.10 baseline CPU, 256 MiB RAM, 5 GiB storage, and small Valkey/Postgres/outbound HTTP usage.
- Resource sample: one p50 note create with index update and a graph-render cache touch.
- Power coefficients: CPU 0.43, memory 0.0021, storage 0.00058, network 0.036.
- Expected p50: 1.9 mWh and 0.0008g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

The provider_sku_pricing binding remains this note/search SKU reference until source data exists in cloud-billing. No invoice, chargeback, or tenant-visible report is changed here.

## RED fixture contract

Future tests must reject rows that price full-text search or graph render without provider/region, and rows that confuse AI assist with a normal note open.
