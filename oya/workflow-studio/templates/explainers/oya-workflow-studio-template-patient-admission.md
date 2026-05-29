# Patient admission -> EHR -> billing pre-auth

**Template id**: `oya-workflow-studio-template-patient-admission`  
**Persona**: `hospital-admissions-clerk`  
**Vertical**: `hospital-operations`  
**Schema version**: `1.0.0`

## What this template does

Admit a patient: registration, insurance verification, EHR record creation, billing pre-authorisation.

## Who uses it

This template is owned by the `hospital-admissions-clerk` persona inside the `hospital-operations` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `patient_mrn` | `reference` | yes | Patient MRN |
| `encounter_type` | `enum` | yes | Encounter type |
| `payer_id` | `reference` | yes | Payer id |

## Node graph

- **Entry**: `trigger-admission-request`
- **Terminals**: `t-admission-complete`
- **Nodes**: 7
- **Edges**: 6

## Connector dependencies

- `oya-shared-connector-payer-eligibility` (required): `verify_eligibility`
- `oya-shared-connector-ehr` (required): `create_patient_encounter`
- `oya-shared-connector-payer-claims` (required): `submit_preauth`

## Compliance flags

`hipaa`, `hitech`, `soc2-type-2`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:patient-admission`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `PatientEncounter`
- **Action**: `admit`
- **Conditions**:
  - `principal.hipaa_role_attested == true`

## SLO

- Max duration: **3600s**
- Min success rate: **0.995**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-patient-admission.openslo.yaml`

## Runtime expectations

- p50: **900s**
- p99: **3600s**

## Cost model (per execution, USD)

- Total (p50): **$1.85**
- Foundry inference: $0.05
- Connector calls: $1.6
- Storage: $0.2

## Audit-chain emission points

- `verify-insurance` -> seal: `external-call-receipt`
- `ehr-register` -> seal: `external-call-receipt`
- `pre-auth` -> seal: `external-call-receipt`
- `audit-admit` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`hospital`, `admission`, `ehr`, `hipaa`

