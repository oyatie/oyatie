# Compensation band refresh cycle

**Template id**: `oya-workflow-studio-template-comp-band-refresh`  
**Persona**: `hr-business-partner`  
**Vertical**: `hr-people`  
**Schema version**: `1.0.0`

## What this template does

Annual comp band refresh: pull market data, run regression, validate against budget, publish bands, notify managers.

## Who uses it

This template is owned by the `hr-business-partner` persona inside the `hr-people` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `fiscal_year` | `string` | yes | Fiscal year code |
| `budget_envelope_pct` | `number` | yes | Budget envelope % |

## Node graph

- **Entry**: `trigger-annual-refresh`
- **Terminals**: `t-bands-published`
- **Nodes**: 8
- **Edges**: 7

## Connector dependencies

- `oya-shared-connector-market-data` (required): `pull_comp_benchmarks`
- `oya-shared-connector-workday` (required): `publish_bands`

## Compliance flags

`soc2-type-2`, `sox-section-404`, `gdpr`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:comp-band-refresh`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `CompBand`
- **Action**: `refresh`

## SLO

- Max duration: **172800s**
- Min success rate: **0.99**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-comp-band-refresh.openslo.yaml`

## Runtime expectations

- p50: **86400s**
- p99: **172800s**

## Cost model (per execution, USD)

- Total (p50): **$6.4**
- Foundry inference: $1.1
- Connector calls: $4.8
- Storage: $0.5

## Audit-chain emission points

- `budget-check` -> seal: `policy-verdict`
- `publish-bands` -> seal: `external-call-receipt`
- `audit-bands` -> seal: `transform-attestation`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`hr`, `compensation`, `annual-cycle`

