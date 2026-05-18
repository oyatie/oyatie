# Interview kit generation + scheduling + debrief

**Template id**: `oya-workflow-studio-template-interview-kit-scheduling`  
**Persona**: `recruiting-coordinator`  
**Vertical**: `hiring`  
**Schema version**: `1.0.0`

## What this template does

Generate interview kit from role spec, schedule loop, send prep, collect scorecards, debrief.

## Who uses it

This template is owned by the `recruiting-coordinator` persona inside the `hiring` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `candidate_id` | `reference` | yes | Candidate id |
| `req_id` | `reference` | yes | Req id |
| `interview_loop_template` | `string` | yes | Loop template code |

## Node graph

- **Entry**: `trigger-candidate-advanced`
- **Terminals**: `t-debrief-complete`
- **Nodes**: 10
- **Edges**: 9

## Connector dependencies

- `oya-shared-connector-calendar` (required): `schedule_loop`
- `oya-shared-connector-ats` (required): `collect_scorecards`

## Compliance flags

`soc2-type-2`, `eeoc`, `gdpr`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:interview-kit-scheduling`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `CandidateInterview`
- **Action**: `schedule_and_debrief`

## SLO

- Max duration: **1209600s**
- Min success rate: **0.99**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-interview-kit-scheduling.openslo.yaml`

## Runtime expectations

- p50: **604800s**
- p99: **1209600s**

## Cost model (per execution, USD)

- Total (p50): **$2.2**
- Foundry inference: $0.85
- Connector calls: $1.2
- Storage: $0.15

## Audit-chain emission points

- `generate-kit` -> seal: `transform-attestation`
- `debrief` -> seal: `human-approval-signature`
- `audit-interview` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`hiring`, `interview`, `scheduling`

