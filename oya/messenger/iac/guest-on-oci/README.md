# OpenTofu Module - messenger / guest-on-oci

Deployment context for tenants hosting messenger inside their own OCI tenancy. Demo trials may use OCI Always Free when `always_free_eligible=true`; paid tenants use standard OKE capacity and customer vault/KMS bindings.

## Contract

- `main.tf` provisions OCI identity policy, the messenger Kubernetes namespace, and the Helm release.
- `versions.tf` pins OpenTofu and provider constraints for OCI, Kubernetes, and Helm.
- Tenant class eligibility is `demo_trial` and `paid`.
- Paid billing components emitted by messenger are `per_seat` and `per_usage`.
- Demo trial caps are enforced by Cedar and cloud-billing, not by reducing messenger feature surface.

## Required Inputs

- `tenant_id`
- `oci_compartment_id`
- `oci_region`
- `oke_cluster_id`
- `tenant_vault_id`
- `tenant_kms_key_id`

## Notes

`always_free_eligible` is valid only for demo_trial tenants. Compliance pack activation requires `tenant_class=paid`.
