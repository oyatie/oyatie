# Vendor onboarding (SOC 2 + DPA + MSA + terms)

**Template id**: `oya-workflow-studio-template-vendor-onboarding`  
**Persona**: `procurement-lead`  
**Vertical**: `operations`  
**Schema version**: `1.0.0`

## What this template does

Onboard a new vendor: SOC 2 attestation collection, DPA signature, MSA negotiation, payment terms registration, vendor master entry.

## Who uses it

This template is owned by the `procurement-lead` persona inside the `operations` vertical.

## Inputs

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `vendor_legal_name` | `string` | yes | Vendor legal name |
| `vendor_taxonomy` | `string` | yes | Vendor taxonomy code |
| `tax_id` | `string` | yes | Vendor tax id |

## Node graph

- **Entry**: `trigger-vendor-request`
- **Terminals**: `t-vendor-active`
- **Nodes**: 9
- **Edges**: 8

## Connector dependencies

- `oya-shared-connector-document-store` (required): `collect_soc2`
- `oya-shared-connector-esign` (required): `send_for_signature`
- `oya-shared-connector-erp` (required): `register_vendor_terms`, `create_vendor`

## Compliance flags

`soc2-type-2`, `gdpr`, `iso-27001`

## Cedar policy

- **Policy id**: `cedar:oya-workflow-studio-template:vendor-onboarding`
- **Effect**: `permit`
- **Principal**: `Workflow::Tenant`
- **Resource**: `Vendor`
- **Action**: `onboard`

## SLO

- Max duration: **1814400s**
- Min success rate: **0.99**
- OpenSLO ref: `microservices/workflow-studio/slos/oya-workflow-studio-template-vendor-onboarding.openslo.yaml`

## Runtime expectations

- p50: **604800s**
- p99: **1814400s**

## Cost model (per execution, USD)

- Total (p50): **$4.6**
- Foundry inference: $0.2
- Connector calls: $3.8
- Storage: $0.6

## Audit-chain emission points

- `soc2-collect` -> seal: `external-call-receipt`
- `dpa-sign` -> seal: `human-approval-signature`
- `msa-negotiate` -> seal: `human-approval-signature`
- `audit-vendor` -> seal: `decision-recorded`

## Test vs live mode

- Test mode supported: **True**
- Live mode supported: **True**

## Tags

`operations`, `vendor`, `procurement`, `soc2`

