# performance-management — oyatie-public-cloud deployment

Oyatie-hosted multi-tenant deployment of `performance-management`.

## Provisioning

```bash
tofu init
tofu plan -var="tenant_id=<uuid>" -var="tenant_class=paid"
tofu apply -var="tenant_id=<uuid>" -var="tenant_class=paid"
```

## Context

- **Cell tier**: T1 default; opt to T2 for SMB.
- **Tenant class**: demo_trial (Always Free if possible) or paid (full surface).
- **Billing**: `bc-performance-management` bound at `billing.tf`.
- **Network**: HTTP/3 + QUIC ingress per `ech-config.yaml` + `pqc-cert.yaml`.

## Files

- `main.tf` — top-level provisioning.
- `versions.tf` — OpenTofu + provider pinning.
- `billing.tf` — billing-component binding.
