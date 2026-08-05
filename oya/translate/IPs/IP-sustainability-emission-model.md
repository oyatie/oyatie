# IP-sustainability-emission-model — translate

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: per_request.

Translate binds emissions to batch translate, document translate, language detection, MT engine invocation, data-residency routing, quality estimation, and auto-translate content-class paths. This advisory block does not claim live provider carbon routing.

## Calibration fixture

- Capacity anchor: `oya/translate/manifest.json` capacity_model uses per_request scaling, 0.35 baseline CPU, 768 MiB RAM, 16 GiB storage, and a high outbound HTTP budget for model/provider calls.
- Resource sample: one p50 document translate page with language detection and provider invocation.
- Power coefficients: CPU 0.66, memory 0.0032, storage 0.00068, network 0.095.
- Expected p50: 9.6 mWh and 0.0038g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

The provider_sku_pricing reference covers translate provider invocation plus document staging after cloud-billing publishes a SKU map. This IP cannot mutate invoices, chargeback, or regulator-export artifacts.

## RED fixture contract

Future validation must reject batch translate, language detection, or document translate rows that omit the five ADR-0344 fields or hide provider/model network cost behind a generic request counter.
