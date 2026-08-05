# IP-sustainability-emission-model — meet

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: media_minutes.

Meet is media-path dominated. The fixture binds to media minute consumption across room creation, SFU path, MLS handshake, live caption, meeting summary, and recording handoff. This declaration does not claim production carbon-aware routing or measured call emissions.

## Calibration fixture

- Capacity anchor: `oya/meet/manifest.json` capacity_model uses per_user scaling, 0.28 baseline CPU, 768 MiB RAM, 4 GiB storage, and outbound HTTP/media control budget.
- Resource sample: one p50 participant media minute with live caption enabled and an MLS handshake amortized across the meeting.
- Power coefficients: CPU 0.69, memory 0.0032, storage 0.00070, network 0.181.
- Expected p50: 42.0 mWh and 0.0168g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

provider_sku_pricing will split compute, media-relay egress, caption, and summary components. This IP is only the plan binding and must not mutate billing or regulator exports.

## RED fixture contract

A future emission test must fail if media minute rows omit watt-hours/CO2/cost/provider/region, or if live caption and meeting summary work disappear into an unpriced availability counter.
