# IP-sustainability-emission-model — mail

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: per_message.

The mail declaration binds the emission model to message delivery paths that already separate inbound SMTP accept, outbound delivery, inbox/thread render, spam classification, retention-policy, legal hold, and e-discovery export SLOs. The baseline fixture is not a measured production claim; it is the RED contract for a future audit-row v2 implementation.

## Calibration fixture

- Capacity anchor: `oya/mail/manifest.json` capacity_model uses per_message scaling, 0.18 baseline CPU, 384 MiB RAM, 25 GiB storage, and Valkey/Postgres/outbound HTTP connections.
- Resource sample: one p50 delivered message with retention metadata and a small search-index update.
- Power coefficients: CPU 0.61, memory 0.0030, storage 0.00120, network 0.083.
- Expected p50: 4.8 mWh and 0.0019g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

Use `provider_sku_pricing` pinned for the mail message-delivery SKU family. The binding remains this IP section until cloud-billing publishes a committed SKU map; no invoice, chargeback, or tenant bill is mutated here.

## RED fixture contract

A future implementation must fail when an audit row omits cost, watt-hours, CO2, provider, or region; when the workload signal is not message-scoped; or when legal hold/e-discovery export is priced as an ordinary inbox read without retention storage.
