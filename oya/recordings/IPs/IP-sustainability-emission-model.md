# IP-sustainability-emission-model — recordings

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: media_minutes.

Recordings binds emissions to recording playback, export MP4, transcript/PDF export, transcription/diarization, auto-translate, legal hold, and chain-of-custody preservation. This is a declaration and fixture plan, not measured media-storage billing.

## Calibration fixture

- Capacity anchor: `oya/recordings/manifest.json` capacity_model uses per_capability scaling, 0.30 baseline CPU, 768 MiB RAM, 50 GiB storage, and media/export outbound HTTP pools.
- Resource sample: one p50 stored media minute with transcript generation and legal hold metadata.
- Power coefficients: CPU 0.74, memory 0.0033, storage 0.00180, network 0.165.
- Expected p50: 56.0 mWh and 0.0224g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

provider_sku_pricing will need media storage, transcode, transcript, and egress SKU splits. This IP is the plan binding; it must not change billing, chargeback, or regulator-export evidence.

## RED fixture contract

Future validation must fail when recording playback, transcript, or legal hold rows lack the five ADR-0344 fields, or when export MP4 work is treated as free metadata.
