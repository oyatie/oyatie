# Month-end close (T-1 to T+5)

**Template id**: `oya-workflow-studio-template-month-end-close`  
**Persona**: `finance-controller`  
**Vertical**: `payroll-finance`  
**Schema version**: `1.0.0`

## What this template does

Close-the-books pipeline: subledger reconciliation, accrual booking, intercompany elimination, FX revaluation, trial balance, lock period.

## Who uses it

This template is owned by the `finance-controller` persona inside the `payroll-finance` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `period_code` | `string` | yes | Accounting period YYYY-MM |
| `entity_set` | `string` | yes | Legal entity set |

## Node graph

- **Entry**: `trigger-close-start`
- **Terminals**: `t-close-complete`
- **Nodes**: 10
- **Edges**: 9

## Connector dependencies

- `oya-shared-connector-erp` (required): `reconcile_subledgers`, `produce_trial_balance`, `lock_period`

## Compliance flags

`soc2-type-2`, `sox-section-404`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:month-end-close`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `AccountingPeriod`
- **Action**: `close_period`

## SLO

- Max duration: **518400s**
- Min success rate: **0.999**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-month-end-close.openslo.yaml`

## Runtime expectations

- p50: **345600s**
- p99: **518400s**

## Cost model (per execution, USD)

- Total (p50): **$14.2**
- Foundry inference: $2.4
- Connector calls: $10.8
- Storage: $1.0

## Audit-chain emission points

- `controller-approval` -> seal: `human-approval-signature`
- `lock-period` -> seal: `external-call-receipt`
- `audit-close` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`finance`, `close`, `sox`

