# Monthly pay run

**Template id**: `oya-workflow-studio-template-monthly-pay-run`  
**Persona**: `payroll-controller`  
**Vertical**: `payroll-finance`  
**Schema version**: `1.0.0`

## What this template does

End-to-end monthly payroll: time-attendance lock, gross-to-net, statutory deductions, ACH batch, payslip emission.

## Who uses it

This template is owned by the `payroll-controller` persona inside the `payroll-finance` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `period_code` | `string` | yes | Pay period code YYYY-MM |
| `jurisdiction` | `string` | yes | Jurisdiction code |

## Node graph

- **Entry**: `trigger-payroll-cutoff`
- **Terminals**: `t-payrun-complete`
- **Nodes**: 9
- **Edges**: 8

## Connector dependencies

- `oya-shared-connector-time-attendance` (required): `lock_period`
- `oya-shared-connector-banking` (required): `submit_ach_batch`
- `oya-shared-connector-document-store` (required): `emit_payslips`

## Compliance flags

`soc2-type-2`, `sox-section-404`, `flsa`, `wage-act`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:monthly-pay-run`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `PayrollPeriod`
- **Action**: `execute_payrun`

## SLO

- Max duration: **21600s**
- Min success rate: **0.9995**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-monthly-pay-run.openslo.yaml`

## Runtime expectations

- p50: **10800s**
- p99: **21600s**

## Cost model (per execution, USD)

- Total (p50): **$24.5**
- Foundry inference: $0.2
- Connector calls: $22.4
- Storage: $1.9

## Audit-chain emission points

- `policy-treasury` -> seal: `policy-verdict`
- `ach-batch` -> seal: `external-call-receipt`
- `audit-payrun` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`payroll`, `finance`, `monthly`

