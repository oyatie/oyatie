# Quarterly variance review with auto-narrative

**Template id**: `oya-workflow-studio-template-quarterly-variance-narrative`  
**Persona**: `finance-controller`  
**Vertical**: `payroll-finance`  
**Schema version**: `1.0.0`

## What this template does

Pull actuals + budget, compute variances, auto-draft narrative via Foundry inference, route to controller for sign-off.

## Who uses it

This template is owned by the `finance-controller` persona inside the `payroll-finance` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `quarter_code` | `string` | yes | Quarter code FYYYYY-Qn |

## Node graph

- **Entry**: `trigger-quarter-end`
- **Terminals**: `t-variance-published`
- **Nodes**: 9
- **Edges**: 8

## Connector dependencies

- `oya-shared-connector-erp` (required): `pull_actuals`
- `oya-shared-connector-fpa` (required): `pull_budget`
- `oya-shared-connector-document-store` (required): `publish_board_pack`

## Compliance flags

`soc2-type-2`, `sox-section-404`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:quarterly-variance-narrative`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `QuarterlyReport`
- **Action**: `publish_variance`

## SLO

- Max duration: **172800s**
- Min success rate: **0.99**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-quarterly-variance-narrative.openslo.yaml`

## Runtime expectations

- p50: **86400s**
- p99: **172800s**

## Cost model (per execution, USD)

- Total (p50): **$5.8**
- Foundry inference: $3.2
- Connector calls: $2.2
- Storage: $0.4

## Audit-chain emission points

- `draft-narrative` -> seal: `transform-attestation`
- `controller-review` -> seal: `human-approval-signature`
- `audit-variance` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`finance`, `variance`, `quarterly`, `narrative`

