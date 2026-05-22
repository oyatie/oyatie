---
doc_class: IaCModuleManifest
microservice: contract-lifecycle-management
deployment_context: oci-guest
profile: always-free
related_memories: [feedback_oci_always_free_maximization_2026_05_20]
date: 2026-05-21
---

# OCI Always Free Module — demo_trial CLM

Provisions a complete CLM demo_trial deployment within OCI Always Free tier (zero cost). Designed for evaluation tenants, side-by-side feature comparison with Ironclad / DocuSign CLM / Conga CLM trials, and migration-tooling validation before paid conversion.

## Always Free envelope used (per tenant)

| OCI resource | Always Free limit | CLM demo_trial usage |
|---|---|---|
| Ampere A1 ARM (4 OCPU + 24 GB total) | 4 OCPU + 24 GB | 1 OCPU + 4 GB |
| Block Volume | 200 GB | 50 GB |
| Autonomous Database | 2× 20 GB | 1× 20 GB |
| Object Storage | 10 GB | 5 GB |
| Egress | 10 TB / month | 10 GB / month (per demo_trial cap) |
| Vault | Yes (Free) | 1 secret keystore |
| Load Balancer | 10 Mbps Always Free | 1 LB |
| Email Delivery | 200 / day | (sufficient for demo) |

Multiple demo_trial tenants share the Always Free envelope on a single OCI tenancy:

- 4 OCPU / 24 GB total → 4 demo_trial tenants of 1 OCPU + 4 GB each.
- 200 GB block → 4 × 50 GB = 200 GB (full).
- 2 × 20 GB Autonomous DB → 2 demo_trial tenants need their own DB; multi-tenant on one DB for 4 tenants.

CLM is the µservice in scope for this module; other µservices (HR, ERP, ITSM) share the remaining envelope per `feedback_oci_always_free_maximization` total-µservice decomposition.

## CLM demo_trial caps enforced (Cedar gates)

- Max 5 active contracts per tenant.
- Max 100 KB per contract document.
- AES e-signature only (no QES; no HSM).
- No AI redlining (Llama not provisioned in this envelope).
- 30-day retention only (not 7y legal floor).
- TEST_DATA data classification (not LEGAL_PRODUCTION).
- No cross-region replication.
- No sovereign-pack overlays (no kr-pipa, no eu-eidas-qes).
- Best-effort SLO (no contractual commitment).

## Module shape

```hcl
module "clm_demo_trial" {
  source = "./modules/clm-demo-trial"

  tenant_id  = var.tenant_id
  oci_region = var.oci_region

  compute = {
    shape   = "VM.Standard.A1.Flex"
    ocpus   = 1
    memory  = 4
  }

  database = {
    cpu_count          = 1
    storage_tb         = 0
    is_free_tier       = true
    is_dedicated       = false
  }

  storage = {
    block_size_gb      = 50
    object_storage_gb  = 5
  }

  network = {
    egress_cap_gb      = 10
  }

  clm_caps = {
    max_active_contracts = 5
    max_doc_size_kb      = 100
    signature_levels     = ["SES", "AES"]
    ai_redlining_enabled = false
    retention_days       = 30
  }
}
```

## Conversion to paid

When a demo_trial tenant converts to `tenant_class=paid`:

1. Trigger the `demo_trial_to_paid` workflow.
2. New OpenTofu apply on the `oci-guest/` (paid) module sub-path with the customer's chosen jurisdiction packs.
3. Existing contracts in the demo_trial state are migrated (re-classified to LEGAL_PRODUCTION, retention extended, packs applied).
4. The Always Free envelope slot is released for the next demo_trial tenant.

## Audit event

`oya.contract.lifecycle.management.tenant.demo_trial.provisioned` with tenant_id, oci_region, Always Free slot id.

`oya.contract.lifecycle.management.tenant.class_converted.demo_trial_to_paid` on conversion.

## Module structure (stub)

```
iac/oci-guest/always-free/
  README.md
  main.tf
  variables.tf
  outputs.tf
  modules/
    clm-demo-trial/
      main.tf
      ampere-a1.tf
      autonomous-free.tf
      block-50g.tf
      object-5g.tf
      vault-free.tf
      cedar-policies.tf
```
