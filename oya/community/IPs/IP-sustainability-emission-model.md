# IP-sustainability-emission-model — community

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: per_request.

Community keeps the Plan/Spec/source-lock boundary: this declaration covers post create, comment, vote, moderation, KB publish, and audit-chain seal planning, but it does not authorize implementation fanout or a live FD-001 product claim.

## Calibration fixture

- Capacity anchor: `oya/community/manifest.json` capacity_model uses per_request scaling, 0.12 baseline CPU, 256 MiB RAM, 12 GiB storage, and modest Valkey/Postgres/outbound HTTP usage.
- Resource sample: one p50 post create with comment-index update, moderation enqueue, and audit seal.
- Power coefficients: CPU 0.51, memory 0.0024, storage 0.00075, network 0.052.
- Expected p50: 2.9 mWh and 0.0012g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

The provider_sku_pricing binding is this IP section until the community SKU map is committed. It must not write billing, chargeback, or regulator-export data.

## RED fixture contract

Future tests must reject a community row that omits post/comment/moderation path identity, treats anonymous subproduct traffic as a standalone manifest claim, or forgets provider and region.
