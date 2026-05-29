# Asset return on offboarding

**Template id**: `oya-workflow-studio-template-asset-return-offboarding`  
**Persona**: `it-asset-admin`  
**Vertical**: `operations`  
**Schema version**: `1.0.0`

## What this template does

Reclaim laptop + peripherals on exit: shipping label, return receipt, wipe + re-image, asset-state update.

## Who uses it

This template is owned by the `it-asset-admin` persona inside the `operations` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `employee_id` | `reference` | yes | Departing employee |
| `asset_ids` | `string` | yes | Comma-separated asset ids |

## Node graph

- **Entry**: `trigger-exit-notified`
- **Terminals**: `t-asset-reclaimed`
- **Nodes**: 9
- **Edges**: 8

## Connector dependencies

- `oya-shared-connector-shipping` (required): `issue_return_label`
- `oya-shared-connector-mdm` (required): `freeze_device`, `wipe_and_reimage`
- `oya-shared-connector-asset-management` (required): `set_state`

## Compliance flags

`soc2-type-2`, `gdpr`, `iso-27001`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:asset-return-offboarding`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `Asset`
- **Action**: `return`

## SLO

- Max duration: **2419200s**
- Min success rate: **0.998**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-asset-return-offboarding.openslo.yaml`

## Runtime expectations

- p50: **1209600s**
- p99: **2419200s**

## Cost model (per execution, USD)

- Total (p50): **$8.4**
- Foundry inference: $0.05
- Connector calls: $1.8
- Storage: $0.2

## Audit-chain emission points

- `freeze-device` -> seal: `external-call-receipt`
- `wipe-reimage` -> seal: `external-call-receipt`
- `audit-return` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`operations`, `it`, `asset`, `offboarding`

