---
doc_class: IaCModuleManifest
microservice: contract-lifecycle-management
deployment_context: oci-guest
related_memories: [feedback_zero_handroll_opentofu_only_2026_05_20, feedback_oci_always_free_maximization_2026_05_20]
date: 2026-05-21
---

# OpenTofu Module — OCI Guest

Deployment context: `oci-guest` (customer's OCI tenancy). Per `feedback_oci_always_free_maximization`, OCI is the canonical free-tier deployment for demo_trial tenants.

## Usage (paid tier)

```bash
tofu apply \
  -var tenant_id=<tenant_id> \
  -var oci_region=<region> \
  -var oci_tenancy_ocid=<ocid> \
  -var jurisdiction_packs='["gdpr", "eidas"]' \
  -var tenant_class=paid \
  -var billing_components='["per_seat"]'
```

## Usage (demo_trial — Always Free)

```bash
tofu apply \
  -var tenant_id=<tenant_id> \
  -var oci_region=ap-seoul-1 \
  -var jurisdiction_packs='["esign"]' \
  -var tenant_class=demo_trial
```

See `oci-guest/always-free/` for the dedicated Always Free module.

## Resources (paid)

- **OKE Cluster** for CLM pods.
- **OCI Object Storage** with Retention Lock in time-bound mode for WORM.
- **OCI Vault** for at-rest encryption + HSM for QES (per `packs/eidas/README.md`).
- **OCI Autonomous Database** (PostgreSQL-compatible) for tenant metadata.
- **OCI Traffic Management** for multi-region routing.
- **OCI Vault Secrets** for credential bindings.
- **VCN + subnets + security lists** per ADR-0253.
- **OCI Logging + OTEL bridge** for observability.

## Resources (Always Free demo_trial)

Per `iac/oci-guest/always-free/`:

- 1× Ampere A1 instance (1 OCPU + 4 GB RAM) for CLM pod.
- 50 GB Block Volume for contract metadata.
- 20 GB Autonomous Database (Always Free tier).
- 10 GB Object Storage.
- 10 GB egress / month (within Always Free 10 TB).

This combination fits within the Always Free envelope and supports demo_trial tenants at zero cost.

## Sovereign-cell variants

For KR sovereign:

```bash
tofu apply \
  -var oci_region=ap-seoul-1 \
  -var jurisdiction_packs='["kr-pipa", "kr-pipa-sovereign"]' \
  -var hsm_qes_required=true
```

## State backend

```hcl
terraform {
  backend "s3" {                         # OCI Object Storage S3-compat
    bucket   = "${var.tenant_id}-tofu-state"
    key      = "clm/oci-guest/state"
    region   = "${var.oci_region}"
    endpoint = "https://${var.oci_namespace}.compat.objectstorage.${var.oci_region}.oraclecloud.com"
    encrypt  = true
  }
}
```

## Module structure (stub)

```
iac/oci-guest/
  main.tf
  variables.tf
  outputs.tf
  versions.tf
  oke.tf
  object-storage.tf
  autonomous-db.tf
  vault.tf
  traffic-management.tf
  vcn.tf
  observability.tf
  always-free/
    README.md
    main.tf
    variables.tf
    outputs.tf
    ampere-a1.tf
    autonomous-free.tf
    block-50g.tf
  sovereign-kr/
    main.tf
    seoul-cell.tf
    busan-replica.tf
  multi-region/
    main.tf
    cross-region-replication.tf
```
