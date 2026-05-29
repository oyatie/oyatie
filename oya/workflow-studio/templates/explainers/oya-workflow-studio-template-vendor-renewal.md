# Vendor renewal (90-day lookback)

**Template id**: `oya-workflow-studio-template-vendor-renewal`  
**Persona**: `procurement-lead`  
**Vertical**: `operations`  
**Schema version**: `1.0.0`

## What this template does

90-day lookback on vendor performance + spend, prep negotiation brief, route to procurement-lead, optionally trigger MSA amendment.

## Who uses it

This template is owned by the `procurement-lead` persona inside the `operations` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `vendor_id` | `reference` | yes | Vendor master id |
| `current_end_date` | `date` | yes | Current contract end date |

## Node graph

- **Entry**: `trigger-renewal-window`
- **Terminals**: `t-renewal-complete`
- **Nodes**: 9
- **Edges**: 10

## Connector dependencies

- `oya-shared-connector-erp` (required): `pull_vendor_spend`
- `oya-shared-connector-vendor-perf` (preferred): `pull_kpis`
- `oya-shared-connector-esign` (preferred): `send_for_signature`

## Compliance flags

`soc2-type-2`, `sox-section-404`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:vendor-renewal`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `VendorContract`
- **Action**: `renew`

## SLO

- Max duration: **1814400s**
- Min success rate: **0.99**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-vendor-renewal.openslo.yaml`

## Runtime expectations

- p50: **604800s**
- p99: **1814400s**

## Cost model (per execution, USD)

- Total (p50): **$2.1**
- Foundry inference: $0.4
- Connector calls: $1.5
- Storage: $0.2

## Audit-chain emission points

- `procurement-review` -> seal: `human-approval-signature`
- `audit-renewal` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`operations`, `vendor`, `renewal`, `procurement`

