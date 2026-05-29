# Equity vesting -> payroll tax event

**Template id**: `oya-workflow-studio-template-equity-vesting-tax-event`  
**Persona**: `payroll-controller`  
**Vertical**: `payroll-finance`  
**Schema version**: `1.0.0`

## What this template does

Equity vesting cliff/tranche triggers FMV capture, payroll tax event (RSU income), tax withholding, ledger entry.

## Who uses it

This template is owned by the `payroll-controller` persona inside the `payroll-finance` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `grant_id` | `reference` | yes | Equity grant id |
| `vest_date` | `date` | yes | Vest date |
| `units_vested` | `number` | yes | Units vested |

## Node graph

- **Entry**: `trigger-vesting-event`
- **Terminals**: `t-tax-event-complete`
- **Nodes**: 7
- **Edges**: 6

## Connector dependencies

- `oya-shared-connector-equity` (required): `capture_fmv`, `sell_to_cover`
- `oya-shared-connector-workday` (required): `post_tax_event`

## Compliance flags

`soc2-type-2`, `sox-section-404`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:equity-vesting-tax-event`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `EquityGrant`
- **Action**: `process_vest_tax`

## SLO

- Max duration: **1200s**
- Min success rate: **0.999**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-equity-vesting-tax-event.openslo.yaml`

## Runtime expectations

- p50: **300s**
- p99: **1200s**

## Cost model (per execution, USD)

- Total (p50): **$0.55**
- Foundry inference: $0.05
- Connector calls: $0.45
- Storage: $0.05

## Audit-chain emission points

- `fmv-capture` -> seal: `external-call-receipt`
- `withholding` -> seal: `external-call-receipt`
- `audit-vesting` -> seal: `transform-attestation`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`payroll`, `equity`, `tax`, `sox`

