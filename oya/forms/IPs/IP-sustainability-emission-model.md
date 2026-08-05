# IP-sustainability-emission-model — forms

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: per_request.

Forms accounts for form build, submission, analytics render, bulk distribute, export CSV, accessibility validation, and AI-assisted form creation as separate request classes. The declaration is not a live telemetry claim.

## Calibration fixture

- Capacity anchor: `oya/forms/manifest.json` capacity_model uses per_request scaling, 0.08 baseline CPU, 256 MiB RAM, 5 GiB storage, and small Valkey/Postgres/outbound HTTP pools.
- Resource sample: one p50 submission with validation, anti-abuse checks, and analytics enqueue.
- Power coefficients: CPU 0.46, memory 0.0023, storage 0.00062, network 0.049.
- Expected p50: 2.1 mWh and 0.0008g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

The provider_sku_pricing reference is the forms submission/analytics SKU family in this IP until a cloud-billing map exists. No billing ledger, invoice, or chargeback mutation is authorized.

## RED fixture contract

Future checks must reject form rows that omit provider/region or price a bulk distribution as a single submission with no recipient fanout component.
