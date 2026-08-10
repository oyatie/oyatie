# iac/oyatie-public-cloud — patient-monitoring

Deployment context: **oyatie-public-cloud**.

Tier mapping: **Tier-2 (city/AZ)** for general RPM / small-network tenants; **Tier-1
(national/regional sovereign)** for federal / DoD / sovereign-EU tenants.

This directory references OpenTofu modules per the global zero-handroll constraint
(`feedback_zero_handroll_opentofu_only_2026_05_20`):

- `module.patient_monitoring_stream_substrate` — gRPC + FlatBuffers + ring buffer
- `module.patient_monitoring_timescaledb` — TimescaleDB cluster with continuous aggregates
- `module.patient_monitoring_clickhouse_cold` — ClickHouse 6-node cluster (cold tier)
- `module.patient_monitoring_postgres_meta` — Postgres-16 for registry + alarm meta
- `module.patient_monitoring_object_storage_waveform` — S3-compatible regional bucket
- `module.patient_monitoring_ml_inference` — LightGBM-rs inference pods
- `module.patient_monitoring_alarm_engine` — smart-alarm engine pods
- `module.patient_monitoring_mobile_notification` — APNs + FCM + WebPush + SMS dispatchers
- `module.patient_monitoring_central_station_backend` — unit-view gRPC server
- `module.patient_monitoring_device_interop` — HL7v2 + IEEE 11073 listeners + vendor
  connectors
- `module.patient_monitoring_rpm_ingest` — wearable webhook + polling
- `module.patient_monitoring_cedar_binding` — pulls bundles from policy-engine µservice
- `module.patient_monitoring_audit_emit` — wires to audit-chain µservice
- `module.patient_monitoring_telemetry_otel` — OpenTelemetry traces + metrics

## Variables

- `tenant_id`
- `cell_id`
- `region` (e.g., us-east-1)
- `kms_key_arn` (per-tenant CMK; see cloud-kms µservice)
- `vpc_id`, `subnet_ids`, `security_group_ids`
- `compliance_packs` (e.g., ["HIPAA-2024", "EU-AI-ACT-2024-HIGH-RISK"])

## Outputs

- `grpc_endpoint`
- `fhir_endpoint`
- `central_station_endpoint`
- `wearable_webhook_endpoint`

## Apply

```bash
tofu init
tofu plan -var-file=tenant.tfvars
tofu apply -var-file=tenant.tfvars
```

## Notes

- HTTP/3 + QUIC enabled by default per ADR-0253.
- Per-tenant KMS keys per ADR-0244.
- Tier-2 cell topology default; Tier-1 toggle via `tier = "tier-1"`.
