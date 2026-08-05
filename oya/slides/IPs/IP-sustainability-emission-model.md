# IP-sustainability-emission-model — slides

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: per_request.

Slides binds emissions to deck open, cell-like edit/render operations, collab cursor sync, CRDT merge, broadcast mode, export PDF, and export MP4 paths. This declaration is a fixture contract only and does not assert measured presentation emissions.

## Calibration fixture

- Capacity anchor: `oya/slides/manifest.json` capacity_model uses per_user scaling, 0.15 baseline CPU, 512 MiB RAM, 12 GiB storage, and media/export outbound HTTP budget.
- Resource sample: one p50 deck open with thumbnail refresh and an export-preview render.
- Power coefficients: CPU 0.56, memory 0.0030, storage 0.00110, network 0.074.
- Expected p50: 8.8 mWh and 0.0035g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

The provider_sku_pricing binding remains this slides deck-render/export SKU reference until cloud-billing publishes source data. No chargeback, invoice, or regulator-export mutation is authorized.

## RED fixture contract

Future validation must reject deck open or export MP4 rows without cost/CO2/watt-hours/provider/region, and reject collab cursor work hidden behind a generic user-seat counter.
