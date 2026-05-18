# Procurement requisition -> PO -> receiving -> invoice match

**Template id**: `oya-workflow-studio-template-procure-to-pay`  
**Persona**: `procurement-lead`  
**Vertical**: `operations`  
**Schema version**: `1.0.0`

## What this template does

P2P pipeline: requisition approval, PO emission, receiving, 3-way invoice match, payment release.

## Who uses it

This template is owned by the `procurement-lead` persona inside the `operations` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `requisition_id` | `reference` | yes | Requisition id |
| `vendor_id` | `reference` | yes | Vendor master id |
| `amount_usd` | `number` | yes | Requisition amount USD |

## Node graph

- **Entry**: `trigger-requisition-submitted`
- **Terminals**: `t-paid`
- **Nodes**: 11
- **Edges**: 10

## Connector dependencies

- `oya-shared-connector-erp` (required): `emit_po`, `record_receiving`
- `oya-shared-connector-banking` (required): `schedule_payment`

## Compliance flags

`soc2-type-2`, `sox-section-404`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:procure-to-pay`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `Requisition`
- **Action**: `approve_and_pay`

## SLO

- Max duration: **3628800s**
- Min success rate: **0.998**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-procure-to-pay.openslo.yaml`

## Runtime expectations

- p50: **1209600s**
- p99: **3628800s**

## Cost model (per execution, USD)

- Total (p50): **$3.6**
- Foundry inference: $0.2
- Connector calls: $2.8
- Storage: $0.6

## Audit-chain emission points

- `approver` -> seal: `human-approval-signature`
- `emit-po` -> seal: `external-call-receipt`
- `payment` -> seal: `external-call-receipt`
- `audit-p2p` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`operations`, `procurement`, `p2p`, `sox`

