# Offer letter -> e-signature -> onboarding handoff

**Template id**: `oya-workflow-studio-template-offer-esign-onboarding-handoff`  
**Persona**: `recruiting-coordinator`  
**Vertical**: `hiring`  
**Schema version**: `1.0.0`

## What this template does

Generate offer letter, comp validation, e-signature, 1-click handoff to new-hire onboarding template.

## Who uses it

This template is owned by the `recruiting-coordinator` persona inside the `hiring` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `candidate_id` | `reference` | yes | Candidate id |
| `req_id` | `reference` | yes | Req id |
| `offer_base_usd` | `number` | yes | Offer base USD |
| `equity_units` | `number` | no | Equity units |
| `start_date` | `date` | yes | Proposed start date |

## Node graph

- **Entry**: `trigger-offer-approved`
- **Terminals**: `t-handoff-complete`
- **Nodes**: 9
- **Edges**: 8

## Connector dependencies

- `oya-shared-connector-esign` (required): `send_for_signature`
- `oya-shared-connector-workflow-bus` (required): `emit_event`

## Compliance flags

`soc2-type-2`, `eeoc`, `gdpr`, `i9-everify`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:offer-esign-onboarding-handoff`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `OfferLetter`
- **Action**: `emit_and_handoff`

## SLO

- Max duration: **1209600s**
- Min success rate: **0.995**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-offer-esign-onboarding-handoff.openslo.yaml`

## Runtime expectations

- p50: **259200s**
- p99: **1209600s**

## Cost model (per execution, USD)

- Total (p50): **$0.85**
- Foundry inference: $0.15
- Connector calls: $0.6
- Storage: $0.1

## Audit-chain emission points

- `comp-validate` -> seal: `policy-verdict`
- `esign` -> seal: `human-approval-signature`
- `audit-offer` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`hiring`, `offer`, `esign`, `handoff`

