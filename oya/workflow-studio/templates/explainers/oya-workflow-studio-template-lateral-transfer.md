# Lateral transfer between teams

**Template id**: `oya-workflow-studio-template-lateral-transfer`  
**Persona**: `hr-business-partner`  
**Vertical**: `hr-people`  
**Schema version**: `1.0.0`

## What this template does

Orchestrate lateral transfer: manager change, cost-centre reassignment, access scope diff, optional relocation.

## Who uses it

This template is owned by the `hr-business-partner` persona inside the `hr-people` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `employee_id` | `reference` | yes | Employee being transferred |
| `new_manager_id` | `reference` | yes | Receiving manager |
| `new_cost_center` | `string` | yes | Receiving cost centre |

## Node graph

- **Entry**: `trigger-transfer-approved`
- **Terminals**: `t-transfer-complete`
- **Nodes**: 8
- **Edges**: 7

## Connector dependencies

- `oya-shared-connector-workday` (required): `reassign_manager`
- `oya-shared-connector-erp` (required): `update_cost_center`
- `oya-shared-connector-identity` (required): `apply_access_diff`

## Compliance flags

`soc2-type-2`, `gdpr`, `iso-27001`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:lateral-transfer`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `EmployeeRecord`
- **Action**: `transfer`

## SLO

- Max duration: **3600s**
- Min success rate: **0.997**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-lateral-transfer.openslo.yaml`

## Runtime expectations

- p50: **900s**
- p99: **2400s**

## Cost model (per execution, USD)

- Total (p50): **$0.32**
- Foundry inference: $0.02
- Connector calls: $0.25
- Storage: $0.05

## Audit-chain emission points

- `manager-change` -> seal: `external-call-receipt`
- `access-apply` -> seal: `policy-verdict`
- `audit-transfer` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`hr`, `transfer`, `access-management`

