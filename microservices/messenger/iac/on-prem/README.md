# OpenTofu Module - messenger / on-prem

Deployment context for customer-owned data centers and sovereign-cell deployments. On-prem messenger is paid-only because it requires enterprise operational handoff, HSM/BYOK integration, and customer site governance.

## Contract

- `main.tf` provisions the messenger namespace and Helm release on a customer Kubernetes cluster.
- `versions.tf` pins OpenTofu and provider constraints for Kubernetes, Helm, and vSphere.
- Tenant class is constrained to `paid`.
- Paid billing components emitted by messenger are `per_seat` and `per_usage`; revenue share may be added only by contract-specific overlay.

## Required Inputs

- `tenant_id`
- `site_id`
- `k8s_cluster_endpoint`
- `k8s_ca_cert`
- `k8s_token`
- `tenant_class`

## Notes

On-prem defaults MLS to `enforce` for sovereign work-mode deployments. Air-gap mode swaps the image repository to the customer registry and blocks external egress.
