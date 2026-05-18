# Voluntary exit + offboarding

**Template id**: `oya-workflow-studio-template-voluntary-exit-offboarding`  
**Persona**: `people-ops-coordinator`  
**Vertical**: `hr-people`  
**Schema version**: `1.0.0`

## What this template does

Employee-initiated resignation: notice period, knowledge transfer, exit interview, final paycheque, COBRA notice.

## Who uses it

This template is owned by the `people-ops-coordinator` persona inside the `hr-people` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `employee_id` | `reference` | yes | Departing employee |
| `last_day` | `date` | yes | Final working day |

## Node graph

- **Entry**: `trigger-resignation-submitted`
- **Terminals**: `t-exit-complete`
- **Nodes**: 9
- **Edges**: 8

## Connector dependencies

- `oya-shared-connector-task-management` (preferred): `create_kt`
- `oya-shared-connector-workday` (required): `final_pay`
- `oya-shared-connector-benefits` (required): `cobra_notice`
- `oya-shared-connector-identity` (required): `deprovision_user`

## Compliance flags

`soc2-type-2`, `gdpr`, `cobra`, `flsa`, `wage-act`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:voluntary-exit-offboarding`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `EmployeeRecord`
- **Action**: `voluntary_exit`

## SLO

- Max duration: **2419200s**
- Min success rate: **0.998**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-voluntary-exit-offboarding.openslo.yaml`

## Runtime expectations

- p50: **1209600s**
- p99: **2419200s**

## Cost model (per execution, USD)

- Total (p50): **$1.6**
- Foundry inference: $0.06
- Connector calls: $1.4
- Storage: $0.14

## Audit-chain emission points

- `final-paycheque` -> seal: `external-call-receipt`
- `cobra-notice` -> seal: `external-call-receipt`
- `audit-exit` -> seal: `termination-record`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`hr`, `offboarding`, `voluntary-exit`, `cobra`

