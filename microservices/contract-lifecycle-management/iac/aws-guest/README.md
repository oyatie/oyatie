---
doc_class: IaCModuleManifest
microservice: contract-lifecycle-management
deployment_context: aws-guest
related_memories: [feedback_zero_handroll_opentofu_only_2026_05_20, feedback_multi_context_provider_agnostic_2026_05_20]
date: 2026-05-21
---

# OpenTofu Module — AWS Guest

Deployment context: `aws-guest` (customer's AWS account; Oyatie deploys CLM µservice but customer owns the AWS billing).

## Usage

```bash
tofu init
tofu apply \
  -var tenant_id=<tenant_id> \
  -var aws_region=<region> \
  -var aws_account_id=<account_id> \
  -var jurisdiction_packs='["gdpr", "esign"]' \
  -var tenant_class=paid \
  -var billing_components='["per_seat"]' \
  -var hsm_byok=false \
  -var hsm_qes_required=false
```

## Resources

- **EKS Cluster** for CLM pods (per ADR-0254 K8s everywhere except edge).
- **S3 bucket** with Object Lock in Compliance mode for WORM (per `legal-dimensions/worm-binding-model.md`).
- **S3 Glacier Vault Lock** for long-archive.
- **RDS PostgreSQL** primary + 2 replicas (tenant metadata).
- **AWS KMS keys** for at-rest encryption.
- **AWS CloudHSM Cluster** (when `hsm_byok=true`).
- **AWS Route 53** for multi-region routing.
- **AWS Secrets Manager** for credential bindings (with OpenBao sidecar bridge).
- **VPC + subnets + security groups** per ADR-0253 transport requirements.
- **CloudWatch + OTEL collector** for observability per ADR-0263.

## Sovereign-cell variants

For EU sovereign deployment:

```bash
tofu apply \
  -var aws_region=eu-central-1 \
  -var jurisdiction_packs='["gdpr", "eidas", "eu-eidas-qes"]' \
  -var hsm_qes_required=true
```

## Cross-region

Multi-region failover module under `aws-guest/multi-region/` provisions cross-region S3 replication + RDS cross-region read replica + Route 53 health checks.

## State backend

Per zero-handroll OpenTofu, state is in customer-owned S3 + DynamoDB locking:

```hcl
terraform {
  backend "s3" {
    bucket         = "${var.tenant_id}-tofu-state"
    key            = "clm/aws-guest/state"
    region         = "${var.aws_region}"
    dynamodb_table = "${var.tenant_id}-tofu-lock"
    encrypt        = true
  }
}
```

## Module signing

All modules signed with Sigstore + cosign + SBOM. Verification:

```bash
oya iac-verify --module aws-guest --signature ./.cosign-sigs/aws-guest.sig
```

## OS support

Supports all 13 OSes from `manifest.json` `supported_oses`. Talos cluster nodes preferred for hardened deployments.

## Module structure (stub; full module pending Wave 15B)

```
iac/aws-guest/
  main.tf
  variables.tf
  outputs.tf
  versions.tf
  eks.tf
  s3.tf
  rds.tf
  kms.tf
  hsm.tf            # conditional on hsm_byok
  route53.tf
  iam.tf
  vpc.tf
  observability.tf
  modules/
    cell/
    sovereign-eu/
    sovereign-kr/
  multi-region/
    main.tf
    cross-region-replication.tf
```
