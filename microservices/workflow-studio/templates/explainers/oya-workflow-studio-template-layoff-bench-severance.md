# Layoff bench + severance + COBRA

**Template id**: `oya-workflow-studio-template-layoff-bench-severance`  
**Persona**: `hr-business-partner`  
**Vertical**: `hr-people`  
**Schema version**: `1.0.0`

## What this template does

Reduction-in-force batch orchestration: WARN-Act notice, severance package, outplacement enrolment, COBRA, audit.

## Who uses it

This template is owned by the `hr-business-partner` persona inside the `hr-people` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `rif_batch_id` | `reference` | yes | RIF batch identifier |
| `effective_date` | `date` | yes | Effective separation date |

## Node graph

- **Entry**: `trigger-rif-approved`
- **Terminals**: `t-rif-complete`
- **Nodes**: 8
- **Edges**: 7

## Connector dependencies

- `oya-shared-connector-document-store` (required): `send_warn_notices`
- `oya-shared-connector-workday` (required): `batch_severance`
- `oya-shared-connector-benefits` (required): `cobra_batch`
- `oya-shared-connector-outplacement` (preferred): `enrol_batch`

## Compliance flags

`soc2-type-2`, `gdpr`, `cobra`, `eeoc`, `flsa`, `wage-act`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:layoff-bench-severance`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `LayoffBatch`
- **Action**: `execute_rif`
- **Conditions**:
  - `resource.warn_act_threshold_satisfied == true`

## SLO

- Max duration: **1814400s**
- Min success rate: **0.999**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-layoff-bench-severance.openslo.yaml`

## Runtime expectations

- p50: **604800s**
- p99: **1814400s**

## Cost model (per execution, USD)

- Total (p50): **$18.4**
- Foundry inference: $0.4
- Connector calls: $16.8
- Storage: $1.2

## Audit-chain emission points

- `policy-rif` -> seal: `policy-verdict`
- `warn-notice` -> seal: `external-call-receipt`
- `audit-rif` -> seal: `termination-record`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`hr`, `rif`, `layoff`, `cobra`, `warn-act`

