# Promotion + compensation change

**Template id**: `oya-workflow-studio-template-promotion-comp-change`  
**Persona**: `hr-business-partner`  
**Vertical**: `hr-people`  
**Schema version**: `1.0.0`

## What this template does

Promotion event: comp band validation, payroll update, equity grant, manager notification, audit.

## Who uses it

This template is owned by the `hr-business-partner` persona inside the `hr-people` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `employee_id` | `reference` | yes | Promoted employee |
| `new_level` | `string` | yes | New level code |
| `new_base_usd` | `number` | yes | New annual base USD |
| `equity_grant_units` | `number` | no | Equity units granted |

## Node graph

- **Entry**: `trigger-promo-approved`
- **Terminals**: `t-promo-complete`
- **Nodes**: 7
- **Edges**: 6

## Connector dependencies

- `oya-shared-connector-workday` (required): `update_compensation`
- `oya-shared-connector-equity` (preferred): `grant_award`

## Compliance flags

`soc2-type-2`, `sox-section-404`, `gdpr`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:promotion-comp-change`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `EmployeeRecord`
- **Action**: `promote`
- **Conditions**:
  - `resource.comp_band_validated == true`

## SLO

- Max duration: **1800s**
- Min success rate: **0.995**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-promotion-comp-change.openslo.yaml`

## Runtime expectations

- p50: **600s**
- p99: **1800s**

## Cost model (per execution, USD)

- Total (p50): **$0.28**
- Foundry inference: $0.03
- Connector calls: $0.2
- Storage: $0.05

## Audit-chain emission points

- `policy-comp-band` -> seal: `policy-verdict`
- `payroll-update` -> seal: `external-call-receipt`
- `equity-grant` -> seal: `external-call-receipt`
- `audit-promo` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`hr`, `promotion`, `compensation`, `equity`

