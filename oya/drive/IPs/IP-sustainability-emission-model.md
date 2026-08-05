# IP-sustainability-emission-model — drive

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: storage_byte_hour.

Drive is storage-heavy: upload, download, preview, sync delta, search, DLP/virus scan, permissions, share links, and immutability tier/retention behavior all affect the emission model. The manifest block is a source declaration only; it is not measured storage billing or live FinOps readiness.

## Calibration fixture

- Capacity anchor: `oya/drive/manifest.json` capacity_model uses per_request scaling with 0.4 baseline CPU, 1024 MiB RAM, 51200 GiB storage, and larger Valkey/Postgres/outbound HTTP pools.
- Resource sample: one p50 25 MiB upload plus thumbnail preview and short-lived immutability metadata write.
- Power coefficients: CPU 0.72, memory 0.0034, storage 0.00165, network 0.142.
- Expected p50: 28.5 mWh and 0.0114g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

provider_sku_pricing will need object storage, scan, preview, and egress SKU components. This IP is the interim binding reference and must not update cloud-billing, invoices, or chargeback allocations.

## RED fixture contract

The future test must fail when upload/download emissions omit storage bytes, network GiB, provider, or region, or when immutability tier retention is collapsed into an ordinary file-read baseline.
