# OpenTofu Module - messenger / colo

Deployment context for customer-owned or rented hardware in colocation facilities. This is a paid-only sovereign deployment path for tenants that need jurisdiction-specific residency with customer-controlled hardware.

## Contract

- `main.tf` provisions the messenger namespace and Helm release on the colocation Kubernetes cluster.
- `versions.tf` pins OpenTofu and provider constraints for Kubernetes and Helm.
- Tenant class is constrained to `paid`.
- Paid billing components emitted by messenger are `per_seat` and `per_usage`; revenue share requires a marketplace or managed-service contract overlay.

## Required Inputs

- `tenant_id`
- `colo_provider`
- `colo_region`
- `k8s_cluster_endpoint`
- `k8s_ca_cert`
- `k8s_token`
- `sovereign_jurisdiction`

## Notes

Compliance pack defaults derive from `sovereign_jurisdiction`. Demo trial tenants cannot use this context because compliance-pack activation and sovereign-cell operations are paid-only.
