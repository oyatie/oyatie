# IP-sustainability-emission-model — tasks

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: per_request.

Tasks maps emissions to task create/update, bulk update, recurring materialise, dependency cycle detection, search, auto-assign fairness checks, and notification fanout. The declaration is not live cost attribution.

## Calibration fixture

- Capacity anchor: `oya/tasks/manifest.json` capacity_model uses per_user scaling, 0.22 baseline CPU, 512 MiB RAM, 10 GiB storage, and outbound HTTP fanout.
- Resource sample: one p50 task mutation with dependency cycle check and recurrence metadata update.
- Power coefficients: CPU 0.47, memory 0.0024, storage 0.00063, network 0.045.
- Expected p50: 2.4 mWh and 0.0010g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

provider_sku_pricing will map task mutation/search/recurrence SKU families after source authority lands. This IP is a service-local reference only and must not mutate billing.

## RED fixture contract

Future tests must fail when recurring materialise or bulk update rows omit provider, region, cost, CO2, or watt-hours, or when auto-assign fairness checks are priced as ordinary reads.
