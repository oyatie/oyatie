# IP-sustainability-emission-model — workflow-studio

Status: Plan/Spec-only. No runtime instrumentation. No billing mutation. No generated JSON hand edits.

## Workload signal source

Signal family: per_request.

Workflow Studio accounts for builder action requests: canvas edits, CRDT merge fanout, DSL emitter/loader actions, policy preview, template marketplace reads, and replay-debugger interactions. The declaration is not evidence that the visual builder emits audit-row v2 fields yet.

## Calibration fixture

- Capacity anchor: `oya/workflow-studio/manifest.json` capacity_model uses per_user scaling, 0.18 baseline CPU, 384 MiB RAM, 4 GiB storage, and outbound HTTP budget for assist/provider calls.
- Resource sample: one p50 canvas node edit with CRDT merge and policy-preview check.
- Power coefficients: CPU 0.57, memory 0.0027, storage 0.00055, network 0.071.
- Expected p50: 3.2 mWh and 0.0013g CO2 at 400 gCO2/kWh, ±20%.

## Provider SKU price binding plan-only

provider_sku_pricing will bind to the workflow-studio builder-action SKU family after cloud-billing source data exists. This IP reference is plan-only and cannot mutate license, billing, or run-history ledgers.

## RED fixture contract

The future test must fail when builder action audit rows do not carry cost/CO2/watt-hours/provider/region or when canvas collaboration is priced as a generic user seat without request evidence.
