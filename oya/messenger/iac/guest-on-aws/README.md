# OpenTofu Module - messenger / guest-on-aws

Deployment context for tenants hosting messenger inside their own AWS account while Oyatie operates the service plane. This path is paid-only in practice because BYOC requires an enterprise contract and customer-owned KMS/BYOK setup.

## Contract

- `main.tf` provisions tenant AWS IAM/KMS bindings, the messenger Kubernetes namespace, and the Helm release.
- `versions.tf` pins OpenTofu and provider constraints for AWS, Kubernetes, and Helm.
- Tenant class is constrained to `paid` for production BYOC. Demo trials should use `guest-on-oci` Always Free or `oyatie-public-cloud`.
- Paid billing components emitted by messenger are `per_seat` and `per_usage`.

## Required Inputs

- `tenant_id`
- `aws_region`
- `vpc_id`
- `private_subnets`
- `eks_cluster_name`
- `tenant_kms_key_id`

## Notes

The module expects AWS provider credentials to assume the tenant-approved operator role. Compliance pack activation remains a paid Cedar-gated operation and is not available to demo_trial tenants.
