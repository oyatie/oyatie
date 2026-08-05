# IP-sustainability-emission-model — social

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: per_request.

Social binds the emission model to feed render, post/reaction write paths, anonymous-mode routing, notification fanout, and moderation classifier activity. This preserves the Draft PRD/source-authority boundary in the manifest: the block is a declaration contract, not a runtime readiness claim.

## Calibration fixture

- Capacity anchor: `oya/social/manifest.json` capacity_model uses per_request scaling, 0.22 baseline CPU, 512 MiB RAM, 20 GiB storage, and a larger Valkey/Postgres/outbound HTTP fanout profile.
- Resource sample: one p50 feed render plus one reaction write and an asynchronous moderation decision enqueue.
- Power coefficients: CPU 0.58, memory 0.0028, storage 0.00105, network 0.078.
- Expected p50: 3.6 mWh and 0.0014g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

Use provider_sku_pricing for the social feed/moderation SKU family once the cloud-billing source map lands. This IP is only a local binding reference today; no chargeback or cost-allocation rows are produced.

## RED fixture contract

A future emission-path test must fail when feed render, reaction, or moderation rows lack the five ADR-0344 fields or when notification fanout is hidden behind a generic request counter.
