# iac/on-prem — patient-monitoring

Deployment context: **on-prem**. The default for ICU/CCU/PACU/ED acute-care
deployments. Operates fully offline for ≥ 24 hours per the patient-safety contract
(per ADR-0332 §G-7).

Tier mapping: Tier-2 cell + Tier-3 edge + Tier-4 in-room sidecar.

## Modules

- `module.patient_monitoring_k3s_cluster` — K3s or Talos K8s on hospital hardware
- `module.patient_monitoring_postgres_primary` — Postgres-16 primary (on-prem disk)
- `module.patient_monitoring_postgres_replica_2` — 2 replicas (synchronous + async)
- `module.patient_monitoring_timescaledb_primary` — TimescaleDB primary + 2 replicas
- `module.patient_monitoring_minio_object_storage` — MinIO S3-compatible on-prem
- `module.patient_monitoring_clickhouse_3node` — ClickHouse 3-node cluster
- `module.patient_monitoring_redpanda_3node` — Redpanda 3-node stream-platform
- `module.patient_monitoring_local_ring_buffer_nvme` — per-host NVMe ring buffer
- `module.patient_monitoring_edge_sidecar_tier3` — Tier-3 edge node
- `module.patient_monitoring_in_room_sidecar_tier4` — Tier-4 in-room device sidecar
- `module.patient_monitoring_cedar_local` — Cedar policy engine on-prem instance
- `module.patient_monitoring_audit_chain_local` — audit-chain local mirror

## Air-gap support

- Optional `--air-gap=true` flag disables outbound internet calls.
- Wearable RPM not supported in pure air-gap (RPM uses bridged outbound).
- ML model updates via signed offline artifact import.

## Variables

- `hospital_id`
- `unit_count`, `bed_count`
- `compliance_packs` (typically HIPAA + FDA 21 CFR + IEC 62304 + ISO 14971)
- `air_gap` (default false)
- `realtime_kernel` (default true for ICU; PREEMPT_RT)

## Apply

```bash
tofu init -backend-config=on-prem-backend.hcl
tofu plan -var-file=hospital.tfvars
tofu apply -var-file=hospital.tfvars
```

## Notes

- BAA executed with hospital prior to apply.
- Per ADR-0254 deployment-model spectrum: on-prem is the canonical
  "customer-substrate" pattern; oyatie operator presence is opt-in via JIT VPN.
- All Tier-1 OSes supported (Talos / RHEL-9 / Oracle-Linux-9 / SUSE-15-SP6 /
  Ubuntu-24.04-LTS) per `supported-oses.json`.
