# Involuntary termination (legal-flagged)

**Template id**: `oya-workflow-studio-template-involuntary-termination`  
**Persona**: `hr-business-partner`  
**Vertical**: `hr-people`  
**Schema version**: `1.0.0`

## What this template does

Legal-gated termination flow: legal review, security access freeze, final compensation, audit + retention.

## Who uses it

This template is owned by the `hr-business-partner` persona inside the `hr-people` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `employee_id` | `reference` | yes | Employee being terminated |
| `termination_reason_code` | `enum` | yes | Termination reason |
| `severance_weeks` | `number` | no | Severance weeks if applicable |

## Node graph

- **Entry**: `trigger-termination-requested`
- **Terminals**: `t-term-complete`
- **Nodes**: 9
- **Edges**: 8

## Connector dependencies

- `oya-shared-connector-identity` (required): `freeze_access`
- `oya-shared-connector-workday` (required): `final_pay_severance`
- `oya-shared-connector-benefits` (required): `cobra_notice`
- `oya-shared-connector-document-store` (required): `retain_with_legal_hold`

## Compliance flags

`soc2-type-2`, `gdpr`, `cobra`, `eeoc`, `flsa`, `wage-act`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:involuntary-termination`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `EmployeeRecord`
- **Action**: `involuntary_terminate`
- **Conditions**:
  - `resource.legal_reviewed == true`
  - `principal.role in ["hr-business-partner"]`

## SLO

- Max duration: **259200s**
- Min success rate: **0.999**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-involuntary-termination.openslo.yaml`

## Runtime expectations

- p50: **86400s**
- p99: **259200s**

## Cost model (per execution, USD)

- Total (p50): **$2.1**
- Foundry inference: $0.1
- Connector calls: $1.7
- Storage: $0.3

## Audit-chain emission points

- `legal-review` -> seal: `human-approval-signature`
- `freeze-access` -> seal: `policy-verdict`
- `audit-term` -> seal: `termination-record`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`hr`, `termination`, `legal-flagged`, `cobra`

