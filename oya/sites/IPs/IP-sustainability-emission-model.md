# IP-sustainability-emission-model — sites

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: per_request.

Sites maps emissions to page render, CMS query, image optimize, static publish, ACME renewal, and accessibility validation requests. The manifest declaration is an advisory plan and not a public hosting carbon claim.

## Calibration fixture

- Capacity anchor: `oya/sites/manifest.json` capacity_model uses per_request scaling, 0.10 baseline CPU, 256 MiB RAM, 10 GiB storage, and CDN/outbound HTTP paths.
- Resource sample: one p50 page render with one optimized image and cached CMS lookup.
- Power coefficients: CPU 0.48, memory 0.0024, storage 0.00070, network 0.069.
- Expected p50: 3.8 mWh and 0.0015g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

provider_sku_pricing will bind to page-render/CDN/image-optimizer SKU components. This IP is the local binding reference and does not touch billing ledgers or regulator exports.

## RED fixture contract

Future checks must fail when page render or image optimize rows omit the five ADR-0344 fields, or when ACME renewal traffic is counted as tenant page views.
