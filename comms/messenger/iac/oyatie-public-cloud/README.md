# OpenTofu Module - messenger / oyatie-public-cloud

Deployment context for Oyatie-operated public-cloud cells. This module owns the messenger namespace, Helm release, certification labels, mobile-app-bundle peer wiring, and Grafana RBAC for managed multi-tenant cells.

## Contract

- `main.tf` provisions the messenger namespace and Helm release.
- `grafana-rbac.tf` provisions the messenger dashboard folder and roles.
- `versions.tf` pins OpenTofu and provider constraints.
- Tenant class eligibility is `demo_trial` and `paid`; the default for hosted public cells is `paid`.
- Paid billing components emitted by messenger are `per_seat` and `per_usage`; demo_trial clears billing components and relies on cloud-billing caps.

## Required Inputs

- `cell_id`
- `tenant_ids`
- `tenant_class`
- `audience_mode`
- `mls_e2ee_mode`
- `compliance_packs`

## Notes

Personal-mode MLS remains default-on. Work-mode MLS remains tenant opt-in through paid compliance-pack activation. The mobile bundle peers are `mail`, `social`, and `community`; clients use one shared cloud-iam session across those panes.
