# Bed availability prediction + transfer orchestration

**Template id**: `oya-workflow-studio-template-bed-availability-prediction`  
**Persona**: `hospital-bed-manager`  
**Vertical**: `hospital-operations`  
**Schema version**: `1.0.0`

## What this template does

Census + LOS prediction, bed-availability projection, inter-unit transfer orchestration, ED hold alerting.

## Who uses it

This template is owned by the `hospital-bed-manager` persona inside the `hospital-operations` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `facility_id` | `reference` | yes | Facility id |
| `horizon_hours` | `number` | yes | Projection horizon hours |

## Node graph

- **Entry**: `trigger-census-pulse`
- **Terminals**: `t-projection-published`
- **Nodes**: 9
- **Edges**: 10

## Connector dependencies

- `oya-shared-connector-ehr` (required): `pull_census`, `initiate_transfer`

## Compliance flags

`hipaa`, `hitech`, `soc2-type-2`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:bed-availability-prediction`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `FacilityCensus`
- **Action**: `project_and_route`

## SLO

- Max duration: **300s**
- Min success rate: **0.998**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-bed-availability-prediction.openslo.yaml`

## Runtime expectations

- p50: **120s**
- p99: **300s**

## Cost model (per execution, USD)

- Total (p50): **$0.18**
- Foundry inference: $0.1
- Connector calls: $0.06
- Storage: $0.02

## Audit-chain emission points

- `predict-los` -> seal: `transform-attestation`
- `project-beds` -> seal: `transform-attestation`
- `audit-bed` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`hospital`, `bed-management`, `prediction`

