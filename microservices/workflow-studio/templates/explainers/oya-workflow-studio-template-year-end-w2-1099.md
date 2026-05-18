# Year-end W-2 + 1099 generation

**Template id**: `oya-workflow-studio-template-year-end-w2-1099`  
**Persona**: `payroll-controller`  
**Vertical**: `payroll-finance`  
**Schema version**: `1.0.0`

## What this template does

Year-end statutory forms: aggregate earnings, validate TINs, generate W-2 / 1099-NEC / 1099-MISC, e-file with IRS + SSA, distribute to recipients.

## Who uses it

This template is owned by the `payroll-controller` persona inside the `payroll-finance` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `tax_year` | `string` | yes | Tax year YYYY |

## Node graph

- **Entry**: `trigger-year-end`
- **Terminals**: `t-forms-distributed`
- **Nodes**: 10
- **Edges**: 9

## Connector dependencies

- `oya-shared-connector-irs` (required): `tin_match`, `efile_forms`
- `oya-shared-connector-ssa` (required): `efile_w2`
- `oya-shared-connector-document-store` (required): `distribute_forms`

## Compliance flags

`soc2-type-2`, `sox-section-404`, `flsa`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:year-end-w2-1099`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `StatutoryForms`
- **Action**: `generate_efile`

## SLO

- Max duration: **518400s**
- Min success rate: **0.9995**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-year-end-w2-1099.openslo.yaml`

## Runtime expectations

- p50: **259200s**
- p99: **518400s**

## Cost model (per execution, USD)

- Total (p50): **$42.0**
- Foundry inference: $1.2
- Connector calls: $38.4
- Storage: $2.4

## Audit-chain emission points

- `validate-tins` -> seal: `external-call-receipt`
- `controller-approval` -> seal: `human-approval-signature`
- `efile-irs` -> seal: `external-call-receipt`
- `audit-y-end` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`payroll`, `year-end`, `w-2`, `1099`, `irs`, `ssa`

