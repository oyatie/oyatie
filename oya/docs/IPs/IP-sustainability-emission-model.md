# IP-sustainability-emission-model — docs

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: per_request.

Docs uses request-scoped accounting for document open, collaborative edit/collab cursor sync, CRDT merge, export PDF, document list, and storage-backed search operations. The declaration stays advisory and does not claim tenant-visible carbon reporting.

## Calibration fixture

- Capacity anchor: `oya/docs/manifest.json` capacity_model uses per_user scaling, 0.12 baseline CPU, 384 MiB RAM, 25 GiB storage, and outbound HTTP budget for export/preview flows.
- Resource sample: one p50 document open with a collab cursor update and one small export PDF preview.
- Power coefficients: CPU 0.53, memory 0.0029, storage 0.00125, network 0.064.
- Expected p50: 7.4 mWh and 0.0030g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

The provider_sku_pricing binding is a docs editor/export SKU family reference in this IP until cloud-billing publishes a path. This does not create invoice, chargeback, or regulator-export evidence.

## RED fixture contract

Future validation must reject rows that treat export PDF as a plain document read, skip storage-byte accounting, or omit provider/region on collaborative edit emissions.
