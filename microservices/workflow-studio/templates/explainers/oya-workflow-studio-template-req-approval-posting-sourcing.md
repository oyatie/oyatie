# Req approval -> posting -> sourcing automation

**Template id**: `oya-workflow-studio-template-req-approval-posting-sourcing`  
**Persona**: `recruiting-coordinator`  
**Vertical**: `hiring`  
**Schema version**: `1.0.0`

## What this template does

Open a req: hiring-manager + finance approval, multi-board posting, automated sourcing pipelines.

## Who uses it

This template is owned by the `recruiting-coordinator` persona inside the `hiring` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `req_id` | `reference` | yes | Req id |
| `role_code` | `string` | yes | Role code |
| `level` | `string` | yes | Level code |

## Node graph

- **Entry**: `trigger-req-drafted`
- **Terminals**: `t-pipeline-active`
- **Nodes**: 8
- **Edges**: 7

## Connector dependencies

- `oya-shared-connector-job-boards` (required): `post_multi_board`
- `oya-shared-connector-ats` (required): `enable_sourcing`

## Compliance flags

`soc2-type-2`, `eeoc`, `gdpr`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:req-approval-posting-sourcing`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `JobRequisition`
- **Action**: `approve_and_post`

## SLO

- Max duration: **604800s**
- Min success rate: **0.995**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-req-approval-posting-sourcing.openslo.yaml`

## Runtime expectations

- p50: **172800s**
- p99: **604800s**

## Cost model (per execution, USD)

- Total (p50): **$1.2**
- Foundry inference: $0.1
- Connector calls: $0.95
- Storage: $0.15

## Audit-chain emission points

- `hm-approval` -> seal: `human-approval-signature`
- `finance-approval` -> seal: `human-approval-signature`
- `audit-req` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`hiring`, `req`, `sourcing`

