# iac/guest-on-oci — patient-monitoring

Deployment context: **guest-on-oci**. Used when the tenant has an OCI account or when
Bronze-tier / demo / sandbox tenants exploit the OCI Always Free tier per the
`feedback_oci_always_free_maximization_2026_05_20` constraint.

Tier mapping: Tier-2 default; Always Free profile available for Bronze tier.

## Modules

- `module.patient_monitoring_oke_cluster` — Oracle Container Engine for Kubernetes
- `module.patient_monitoring_autonomous_db` — Autonomous Database (Postgres-compatible)
- `module.patient_monitoring_block_volume_waveform` — block storage for hot tier
- `module.patient_monitoring_object_storage_oci` — OCI Object Storage
- `module.patient_monitoring_streaming_oci` — OCI Streaming (Kafka-compatible)
- `module.patient_monitoring_vault_secrets` — OCI Vault for KMS
- `module.patient_monitoring_load_balancer_oci` — OCI Load Balancer
- `module.patient_monitoring_notification_oci` — OCI Notifications

## Always Free profile (Bronze tier)

`iac/guest-on-oci/always-free/` exploits:

- 2× Ampere A1 ARM 4 OCPU + 24 GB instances (host the Rust binaries)
- 2× Autonomous Database (registry + alarm meta)
- 200 GB block volume (waveform hot tier)
- 10 GB object storage (FHIR archive overflow)
- 10 TB egress (RPM patient portal traffic)
- 1× OCI Vault
- 1× Load Balancer
- OCI Streaming basic tier

Bronze tier supports demo + sandbox + trial + dev tenants; ≤ 50 active RPM patients per
tenant; ICU/CCU bedside monitoring not supported on Always Free (insufficient durability
guarantees per the HIPAA + IEC 62304 SaMD Class C posture).

## Variables

- `tenant_ocid`
- `region` (e.g., us-ashburn-1, ap-seoul-1)
- `compartment_ocid`
- `compliance_packs`

## Apply

```bash
tofu init
tofu plan -var-file=tenant-oci.tfvars
tofu apply -var-file=tenant-oci.tfvars
```
