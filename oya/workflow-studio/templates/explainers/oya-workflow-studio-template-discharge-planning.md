# Discharge planning

**Template id**: `oya-workflow-studio-template-discharge-planning`  
**Persona**: `hospital-discharge-planner`  
**Vertical**: `hospital-operations`  
**Schema version**: `1.0.0`

## What this template does

Discharge orchestration: transport, follow-up appointment, prescription, home health, family notification.

## Who uses it

This template is owned by the `hospital-discharge-planner` persona inside the `hospital-operations` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `encounter_id` | `reference` | yes | Encounter id |
| `discharge_disposition` | `enum` | yes | Discharge disposition |

## Node graph

- **Entry**: `trigger-discharge-ordered`
- **Terminals**: `t-discharge-complete`
- **Nodes**: 9
- **Edges**: 8

## Connector dependencies

- `oya-shared-connector-medical-transport` (preferred): `book_transport`
- `oya-shared-connector-ehr` (required): `schedule_followup`
- `oya-shared-connector-pharmacy` (required): `eprescribe`
- `oya-shared-connector-home-health` (preferred): `refer_patient`

## Compliance flags

`hipaa`, `hitech`, `soc2-type-2`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:discharge-planning`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `PatientEncounter`
- **Action**: `discharge`
- **Conditions**:
  - `principal.hipaa_role_attested == true`

## SLO

- Max duration: **21600s**
- Min success rate: **0.995**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-discharge-planning.openslo.yaml`

## Runtime expectations

- p50: **7200s**
- p99: **21600s**

## Cost model (per execution, USD)

- Total (p50): **$2.4**
- Foundry inference: $0.1
- Connector calls: $2.1
- Storage: $0.2

## Audit-chain emission points

- `prescription` -> seal: `external-call-receipt`
- `home-health` -> seal: `external-call-receipt`
- `audit-discharge` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`hospital`, `discharge`, `ehr`, `hipaa`

