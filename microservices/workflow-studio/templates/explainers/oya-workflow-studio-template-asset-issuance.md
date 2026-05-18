# Asset issuance (new hire kit)

**Template id**: `oya-workflow-studio-template-asset-issuance`  
**Persona**: `it-asset-admin`  
**Vertical**: `operations`  
**Schema version**: `1.0.0`

## What this template does

Issue laptop + peripherals + accounts + access for a new hire; track shipment + first-boot enrolment.

## Who uses it

This template is owned by the `it-asset-admin` persona inside the `operations` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `employee_id` | `reference` | yes | Hire ref |
| `kit_profile` | `enum` | yes | Kit profile |
| `ship_address_ref` | `reference` | yes | Ship-to address ref |

## Node graph

- **Entry**: `trigger-hire-confirmed`
- **Terminals**: `t-asset-active`
- **Nodes**: 8
- **Edges**: 7

## Connector dependencies

- `oya-shared-connector-asset-management` (required): `allocate_kit`
- `oya-shared-connector-shipping` (required): `ship_kit`
- `oya-shared-connector-mdm` (required): `enrol_device`
- `oya-shared-connector-identity` (required): `create_accounts`

## Compliance flags

`soc2-type-2`, `iso-27001`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:asset-issuance`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `Asset`
- **Action**: `issue`

## SLO

- Max duration: **259200s**
- Min success rate: **0.998**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-asset-issuance.openslo.yaml`

## Runtime expectations

- p50: **86400s**
- p99: **259200s**

## Cost model (per execution, USD)

- Total (p50): **$38.4**
- Foundry inference: $0.05
- Connector calls: $2.4
- Storage: $0.15

## Audit-chain emission points

- `inventory-allocate` -> seal: `external-call-receipt`
- `mdm-enrol` -> seal: `external-call-receipt`
- `audit-asset` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`operations`, `it`, `asset`, `onboarding`

