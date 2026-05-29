# Performance Improvement Plan (PIP)

**Template id**: `oya-workflow-studio-template-performance-improvement-plan`  
**Persona**: `hr-business-partner`  
**Vertical**: `hr-people`  
**Schema version**: `1.0.0`

## What this template does

Initiate, track, and conclude a PIP with weekly check-ins, HR sign-off, and outcome routing.

## Who uses it

This template is owned by the `hr-business-partner` persona inside the `hr-people` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `employee_id` | `reference` | yes | Employee on PIP |
| `duration_days` | `number` | yes | PIP duration |
| `goals` | `string` | yes | Documented PIP goals |

## Node graph

- **Entry**: `trigger-pip-initiated`
- **Terminals**: `t-pip-success`, `t-pip-fail`
- **Nodes**: 9
- **Edges**: 8

## Connector dependencies

- `oya-shared-connector-workday` (required): `log_pip`
- `oya-shared-connector-document-store` (required): `store_pip_record`

## Compliance flags

`soc2-type-2`, `gdpr`, `eeoc`, `flsa`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:performance-improvement-plan`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `EmployeeRecord`
- **Action**: `initiate_pip`
- **Conditions**:
  - `resource.legal_reviewed == true`

## SLO

- Max duration: **5443200s**
- Min success rate: **0.999**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-performance-improvement-plan.openslo.yaml`

## Runtime expectations

- p50: **5184000s**
- p99: **5443200s**

## Cost model (per execution, USD)

- Total (p50): **$1.2**
- Foundry inference: $0.18
- Connector calls: $0.85
- Storage: $0.17

## Audit-chain emission points

- `hr-sign-off` -> seal: `human-approval-signature`
- `audit-pip-pass` -> seal: `decision-recorded`
- `audit-pip-fail` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`hr`, `performance`, `pip`, `legal-flagged`

