---
doc_class: IaCModuleManifest
microservice: contract-lifecycle-management
deployment_context: oyatie-as-cloud-provider
related_memories: [feedback_zero_handroll_opentofu_only_2026_05_20, feedback_multi_context_provider_agnostic_2026_05_20]
date: 2026-05-21
---

# OpenTofu Module — Oyatie as Cloud Provider

Deployment context: `oyatie-as-cloud-provider` (Oyatie operates its own IaaS / PaaS surface; downstream tenants deploy through Oyatie's cloud as their cloud provider, not through AWS / OCI / Azure / GCP).

This is the "Oyatie cloud" deployment. Oyatie itself provides the underlying IaaS surface; CLM is a tenant-deployed µservice using Oyatie's IaaS.

## Usage

```bash
tofu apply \
  -var tenant_id=<tenant_id> \
  -var oyatie_region=<region> \
  -var oyatie_account_id=<account_id> \
  -var jurisdiction_packs='["gdpr", "esign"]' \
  -var tenant_class=paid \
  -var billing_components='["per_seat"]'
```

## Resources

- **Oyatie K8s cluster** (Cloud Hypervisor + Kata pods per ADR-0254).
- **Oyatie Object Storage** with WORM compliance.
- **Oyatie Managed PostgreSQL** for tenant metadata.
- **Oyatie KMS** for at-rest encryption.
- **Oyatie HSM-as-a-Service** for QES (when sovereign-cell + per pack).
- **Oyatie Load Balancer** for ingress.
- **Oyatie OTEL Collector** for observability.

## Cloud-* µservice integration

Per the `oyatie-as-cloud-provider` context, CLM consumes Oyatie's own cloud-* µservices:

- `cloud-compute-functions-api` for serverless workers.
- `cloud-compute-k8s-api` for orchestration.
- `cloud-data-kernel` for storage.
- `cloud-kms` for encryption keys + HSM.
- `cloud-iam` for principal authentication.
- `cloud-finops-api` for cost tracking.
- `cloud-billing-tax-app` for billing.

## State backend

```hcl
terraform {
  backend "s3" {                       # Oyatie's S3-compatible object storage
    bucket   = "${var.tenant_id}-tofu-state"
    key      = "clm/oyatie-cloud-provider/state"
    region   = "${var.oyatie_region}"
    endpoint = "https://obj.${var.oyatie_region}.cloud.oyatie.dev"
    encrypt  = true
  }
}
```

## Module structure (stub)

```
iac/oyatie-cloud-provider/
  README.md
  main.tf
  variables.tf
  outputs.tf
  versions.tf
  k8s.tf
  object-storage.tf
  postgres.tf
  kms.tf
  hsm-aas.tf
  load-balancer.tf
  observability.tf
  multi-region/
    main.tf
```
