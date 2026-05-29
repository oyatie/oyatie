# Joint Commission preparation checklist

**Template id**: `oya-workflow-studio-template-joint-commission-prep`  
**Persona**: `hospital-compliance-officer`  
**Vertical**: `hospital-operations`  
**Schema version**: `1.0.0`

## What this template does

Surface readiness deficits across patient safety, infection control, medication management, documentation; assign owners + deadline.

## Who uses it

This template is owned by the `hospital-compliance-officer` persona inside the `hospital-operations` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `facility_id` | `reference` | yes | Facility id |
| `cycle_start_date` | `date` | yes | Cycle start date |

## Node graph

- **Entry**: `trigger-jc-cycle`
- **Terminals**: `t-jc-ready`
- **Nodes**: 10
- **Edges**: 9

## Connector dependencies

- `oya-shared-connector-ehr` (required): `pull_incidents`, `pull_infection_logs`, `pull_medication_logs`
- `oya-shared-connector-task-management` (required): `assign_tasks`

## Compliance flags

`hipaa`, `hitech`, `joint-commission`, `soc2-type-2`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:joint-commission-prep`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `FacilityReadiness`
- **Action**: `prepare_jc`

## SLO

- Max duration: **518400s**
- Min success rate: **0.99**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-joint-commission-prep.openslo.yaml`

## Runtime expectations

- p50: **259200s**
- p99: **518400s**

## Cost model (per execution, USD)

- Total (p50): **$7.4**
- Foundry inference: $1.2
- Connector calls: $5.6
- Storage: $0.6

## Audit-chain emission points

- `compile-deficits` -> seal: `transform-attestation`
- `compliance-review` -> seal: `human-approval-signature`
- `audit-jc` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`hospital`, `compliance`, `joint-commission`

