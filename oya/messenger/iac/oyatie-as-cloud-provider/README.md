# OpenTofu Module - messenger / oyatie-as-cloud-provider

Deployment context for tenants consuming Oyatie as the cloud provider. The messenger workload runs on Oyatie-owned cloud substrate and emits usage into cloud-billing alongside compute, storage, and network consumption.

## Contract

- `main.tf` provisions the messenger namespace and Helm release on the Oyatie cloud cell.
- `versions.tf` pins OpenTofu and provider constraints for Kubernetes and Helm.
- Tenant class eligibility is `demo_trial` and `paid`.
- Paid billing components emitted by messenger are `per_seat` and `per_usage`.
- Demo trial tenants receive the same capability surface with caps on message volume, channel count, huddle minutes, attachment storage, and retention.

## Required Inputs

- `tenant_id`
- `oyatie_cell_id`
- `k8s_cluster_endpoint`
- `k8s_ca_cert`
- `k8s_token`
- `tenant_class`

## Notes

This module is the canonical path for Oyatie-as-cloud-provider trials and paid tenants. Compliance pack activation remains denied for demo_trial and permitted only for paid tenants with the required Cedar/billing claims.
