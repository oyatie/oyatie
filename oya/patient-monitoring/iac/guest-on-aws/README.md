# iac/guest-on-aws — patient-monitoring

Deployment context: **guest-on-aws**. Used when the tenant has an existing AWS contract
and prefers to host oyatie patient-monitoring inside their AWS account (per-tenant VPC).

Tier mapping: Tier-1 or Tier-2; per ADR-0248. The default is Tier-2 with EKS cluster
hosted in the tenant's account, peered to oyatie's control plane.

## Modules

- Same module set as `oyatie-public-cloud/` but with `provider = "aws-guest"`.
- `module.patient_monitoring_eks_cluster` — EKS cluster (Talos-on-EKS or Amazon Linux)
- `module.patient_monitoring_rds_postgres` — RDS Postgres-16 (instead of self-managed)
- `module.patient_monitoring_rds_timescale` — RDS with TimescaleDB extension
- `module.patient_monitoring_msk_kafka` — MSK for stream-platform binding (optional;
  default uses oyatie central stream-platform)
- `module.patient_monitoring_s3_waveform` — S3 bucket per tenant
- `module.patient_monitoring_kms_cmk` — per-tenant CMK in tenant account
- `module.patient_monitoring_sns_sqs_notification` — SNS + SQS for mobile-notification
  routing
- `module.patient_monitoring_route53_endpoint` — Route53 private DNS

## Variables

- `tenant_account_id`
- `cross_account_role_arn` (oyatie operator role for tenant account)
- `vpc_cidr`, `availability_zones`
- `compliance_packs`

## Apply

```bash
AWS_PROFILE=tenant-cross-account tofu init
tofu plan -var-file=tenant-aws.tfvars
tofu apply -var-file=tenant-aws.tfvars
```

## Notes

- BAA executed with tenant's AWS account per HIPAA prior to apply.
- VPC peering or PrivateLink for stream-platform binding.
- Per ADR-0254 deployment-model spectrum: guest-on-aws is a "borrowed-substrate"
  pattern with oyatie operator presence via cross-account IAM.
