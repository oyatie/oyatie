# iac/oyatie-as-cloud-provider — patient-monitoring

Deployment context: **oyatie-as-cloud-provider**. The fully oyatie-operated cellular
substrate. oyatie acts as the IaaS + PaaS + SaaS layer for tenants who want a complete
turnkey patient-monitoring SaaS.

Tier mapping: Tier-1 sovereign hosts for EU + KR + DoD; Tier-2 for general.

## Modules

- `module.patient_monitoring_oyatie_cell_tier1` — sovereign cell module
- `module.patient_monitoring_oyatie_cell_tier2` — city/AZ cell module
- `module.patient_monitoring_oyatie_cloud_hypervisor_pool` — Cloud Hypervisor + Kata
  per ADR-0254
- `module.patient_monitoring_oyatie_k8s_pool` — K8s control plane managed by oyatie
- `module.patient_monitoring_oyatie_postgres_managed` — managed Postgres-16
- `module.patient_monitoring_oyatie_timescale_managed` — managed Timescale
- `module.patient_monitoring_oyatie_clickhouse_managed` — managed ClickHouse
- `module.patient_monitoring_oyatie_kms_global` — global KMS escrow per ADR-0248 Tier-0
- `module.patient_monitoring_oyatie_audit_chain_global` — global audit-chain root
- `module.patient_monitoring_oyatie_compliance_pack_engine` — pack-overlay engine

## Variables

- `tenant_id`
- `tier` (tier-1 or tier-2)
- `cell_count`
- `sovereign_overlay` (true for sovereign hosts)
- `compliance_packs`

## Apply

```bash
tofu init
tofu plan -var-file=tenant.tfvars
tofu apply -var-file=tenant.tfvars
```

## Notes

- This is the fully-managed "oyatie SaaS" path. Tenants pay billing components
  enumerated in `manifest.json` `paid_billing_components_emitted`.
- Per global memory: oyatie acts as the cloud provider; cloud-* µservices ARE oyatie's
  own IaaS surface, not AWS/OCI wrappers.
- Tier-1 sovereign cells use dedicated hardware + HSM + air-gapped administrative
  access.
