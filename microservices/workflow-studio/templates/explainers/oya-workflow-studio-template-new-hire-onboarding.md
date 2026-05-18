# New hire onboarding (Day 0 to Day 30)

**Template id**: `oya-workflow-studio-template-new-hire-onboarding`  
**Persona**: `hr-business-partner`  
**Vertical**: `hr-people`  
**Schema version**: `1.0.0`

## What this template does

Day-0 through Day-30 onboarding pipeline: identity provisioning, payroll registration, asset issuance, training enrolment, and Day-30 sentiment check.

## Who uses it

This template is owned by the `hr-business-partner` persona inside the `hr-people` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `employee_id` | `reference` | yes | Employee reference issued at offer signing |
| `start_date` | `date` | yes | First day of employment |
| `role_code` | `string` | yes | Role classification code |

## Node graph

- **Entry**: `trigger-offer-signed`
- **Terminals**: `t-day30-complete`
- **Nodes**: 10
- **Edges**: 9

## Connector dependencies

- `oya-shared-connector-identity` (required): `provision_user`
- `oya-shared-connector-workday` (required): `hire_employee`
- `oya-shared-connector-asset-management` (required): `issue_kit`
- `oya-shared-connector-lms` (preferred): `enrol_curriculum`
- `oya-shared-connector-survey` (preferred): `send_survey`

## Compliance flags

`soc2-type-2`, `gdpr`, `ccpa`, `i9-everify`, `iso-27001`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:new-hire-onboarding`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `EmployeeRecord`
- **Action**: `onboard`
- **Conditions**:
  - `principal.role in ["hr-business-partner", "people-ops-coordinator"]`

## SLO

- Max duration: **2851200s**
- Min success rate: **0.995**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-new-hire-onboarding.openslo.yaml`

## Runtime expectations

- p50: **2592000s**
- p99: **2851200s**

## Cost model (per execution, USD)

- Total (p50): **$2.85**
- Foundry inference: $0.04
- Connector calls: $2.4
- Storage: $0.41

## Audit-chain emission points

- `identity-provision` -> seal: `external-call-receipt`
- `payroll-register` -> seal: `external-call-receipt`
- `audit-onboarding` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`hr`, `onboarding`, `day-0`, `day-30`, `lifecycle`

