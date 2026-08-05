# IP-sustainability-emission-model — calendar

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: per_request.

Calendar uses request-scoped accounting because the emission-relevant paths are event create/update, attendee fanout, free/busy queries, ICS import, and reminder notification fanout. The manifest block is an advisory declaration only; it does not claim live carbon-aware scheduling or production cost readiness.

## Calibration fixture

- Capacity anchor: `oya/calendar/manifest.json` capacity_model uses per_request scaling, 0.08 baseline CPU, 192 MiB RAM, 3 GiB storage, and small Valkey/Postgres/outbound HTTP pools.
- Resource sample: one p50 event create with 10 attendees, a free/busy cache touch, and one reminder enqueue.
- Power coefficients: CPU 0.44, memory 0.0022, storage 0.00045, network 0.041.
- Expected p50: 1.7 mWh and 0.0007g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

Use the calendar request/reminder SKU family once cloud-billing publishes it; until then this service-local IP is the provider_sku_pricing binding reference and no billing ledger is changed.

## RED fixture contract

The future test must reject a row that treats reminder delivery as storage-only, drops region/provider, or collapses free/busy fanout into an unscoped calendar event counter.
