# Patient Monitoring Remediation Notes - 2026-05-21

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- No rewrite required; the service had no Redis vocabulary in the Wave 15-Valkey inventory.

Counterpart-fact preservations:
- None.

Files renamed:
- None.
## Wave 15-doctrine-propagation-IPs (2026-05-21)

- Bucket: D4-BUCKET-5.
- Agent: wave-d-d4-bucket-5-codex.
- Scope: trigger-based doctrine propagation only; unmatched IPs were left unchanged.
- IPs scanned: 10.
- Trigger A matched: 1.
- Trigger B matched: 9.
- Trigger C matched: 0.
- Trigger D matched: 0.
- IPs unmatched: 1.

### IP changes
- `microservices/patient-monitoring/implementation-plans/IP-001-streaming-substrate-grpc-flatbuffers.md` — added API Versioning, DR posture.
- `microservices/patient-monitoring/implementation-plans/IP-003-smart-alarm-engine.md` — added DR posture.
- `microservices/patient-monitoring/implementation-plans/IP-004-mobile-notification-dispatch.md` — added DR posture.
- `microservices/patient-monitoring/implementation-plans/IP-005-central-station-render.md` — added DR posture.
- `microservices/patient-monitoring/implementation-plans/IP-006-ml-inference-deterioration-sepsis.md` — added DR posture.
- `microservices/patient-monitoring/implementation-plans/IP-007-code-blue-activation-playback.md` — added DR posture.
- `microservices/patient-monitoring/implementation-plans/IP-008-rpm-wearable-integration.md` — added DR posture.
- `microservices/patient-monitoring/implementation-plans/IP-009-waveform-archive-tiered-storage.md` — added DR posture.
- `microservices/patient-monitoring/implementation-plans/IP-010-icu-bundle-compliance.md` — added DR posture.

### Unmatched IPs
- `microservices/patient-monitoring/implementation-plans/IP-002-device-interop-hl7v2-ieee11073.md` — no trigger match; no doctrine section added.

### Follow-up
- `microservices/patient-monitoring/manifest.json#dr` is absent; DR sections use `specs/compliance-pack-floors.json` floors and must be reconciled when the D-2 manifest DR block lands.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- Bucket: D3-BUCKET-12.
- Scope: PRD doctrine propagation for `patient-monitoring`; PRD values match the present `manifest.json` `dr` and `capacity_model` blocks.

### DR posture
- Values: RTO 30s, RPO 1s, active-active acute-care streams, failover_runbook `microservices/patient-monitoring/runbooks/patient-monitoring-cell-failover.md`.
- ADR: ADR-0343; HIPAA and EU-AI high-risk floors are looser than the clinical-realtime target.
- Alternative considered: archive-first recovery; rejected because alarms, waveforms, and code-blue activation are live safety state.
- Cost: requires hot replication and standby stream capacity rather than cold archive-only durability.

### Capacity model
- Values: 6.0 vCPU, 8192 MiB RAM, 2048 GB storage, 12 Postgres connections, 4 Valkey connections, 24 outbound HTTP connections; `per_message` scaling; Tier-2 placement; 4-80 pods per tenant cell.
- ADR: ADR-0340.
- Alternative considered: per-user scaling; rejected because vital, waveform, alarm, HL7, and IEEE 11073 frames drive load more than human users.
- Cost: creates high baseline stream and storage reservations for telemetry-heavy tenants.

### Sustainability + cost attribution
- Values: audit rows carry `cost_usd_minor_units`, `co2_grams`, and `watt_hours`; carbon routing disabled for acute telemetry, alarms, code-blue, EU-AI high-risk deterioration, and HIPAA emergency paths.
- ADR: ADR-0344; ADR-0337 applies to non-urgent TrendAnalytics and data-warehouse publication.
- Alternative considered: carbon-route waveform compaction and live telemetry together; rejected because live telemetry has clinical latency invariants.
- Cost: adds per-bed, per-device, per-channel, and per-archive-tier cost/cardinality.

### API versioning
- Values: YYYY-MM-DD carrier triplet, SDK semver, last 3 versions for at least 180 days, tenant pinning for device, central-station, wearable, and FHIR/AsyncAPI consumers, internal-mesh exemption.
- ADR: ADR-0342.
- Alternative considered: device-vendor versioning only; rejected because hospital tenants need a stable Oyatie contract independent of device fleet churn.
- Cost: maintains compatibility across streaming, mobile, FHIR, and wearable client families.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 6 vCPU, 8192 MiB RAM, 2048 GB storage, and per_message scaling follow the documented 100k gRPC streams, vital samples, waveform samples, HL7 messages, and IEEE 11073 frames per cell.
- ADR: ADR-0340 capacity envelopes and ADR-0340 D-6 pod-runtime/cell-placement covariance.
- Rejected: Rejected Tier-3 placement because pod_runtime_tier=1 cannot co-vary with Tier-3, even though edge acquisition has Tier-3 preflight behavior.
- Cost: Commits Patient Monitoring to expensive hot telemetry buffers, object-store replication, and Kata-backed nodes for clinical alarm continuity.

### Block 2: dr
- Values: RTO 30s, RPO 1s, active-active true, backup substrates postgres_wal_g, object_storage_versioned, clickhouse_iceberg_layered, audit_chain_merkle_seal.
- ADR: ADR-0343 recoverability floors, with compliance-pack floors treated as minimums.
- Rejected: Rejected HIPAA floor RTO/RPO because losing bedside alarm or waveform continuity for minutes is clinically unacceptable.
- Cost: Commits the service to runbook-backed failover drills and evidence capture at runbooks/patient-monitoring-cell-failover.md.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=1; Patient Monitoring is a life-critical PHI telemetry and alarm data-plane service with continuous physiologic streams, bedside buffers, and clinical alerting. It does not execute tenant-customer code, but its tenant data-plane blast radius requires Tier 1 isolation and Tier-2 cell placement rather than the Tier-3 edge preflight lane.
- ADR: ADR-0338 pod runtime tiering and ADR-0340 D-6 covariance.
- Rejected: Rejected Tier 0 because the service runs first-party telemetry/alarm logic, not tenant-uploaded code.
- Cost: Commits placement and scheduling to the declared runtime isolation class rather than cheapest generic app placement.

### Block 4: tenant_version_pinning
- Values: declared version 2026-05-21, default 2026-05-21, three-version support window, 180 day minimum support, per-tenant pinning enabled.
- ADR: ADR-0342 tenant/API version pinning and manifest schema public_surface_files contract map.
- Rejected: Rejected synthetic historical API dates because only current public contract files are present.
- Cost: Future contract changes need explicit version calendars and migration documents before tenant sunset.

### Block 5: consumes_upstream_oss
- Values: postgresql, kafka, clickhouse, iceberg, cedar, opentofu.
- ADR: ADR-0345 OSS stewardship declarations, using registry dep_name strings from specs/oss-stewardship-registry.json.
- Rejected: Rejected Valkey because the inspected PRD/architecture evidence points to Postgres, Kafka, ClickHouse/Iceberg, and object storage instead.
- Cost: CVE response ownership and upgrade stewardship now attach to the declared upstream substrate set.

### Block 6: iac_module_invocations
- Values: aws-guest/tenant-namespace, aws-guest/postgres-wal-g, aws-guest/object-storage-versioned, aws-guest/kafka-cluster, oci-guest/tenant-namespace, oci-guest/object-storage-versioned, on-prem/tenant-namespace, colo/tenant-namespace, oyatie-as-cloud-provider/per-cell-nodepool-kata, oyatie-as-cloud-provider/shard-cell.
- ADR: ADR-0339 shared IaC module invocation doctrine and manifest schema authority.
- Rejected: Rejected oyatie-public-cloud as a manifest context spelling because schema authority uses oyatie-as-cloud-provider.
- Cost: Provider-specific IaC must remain a thin invocation layer over shared module primitives and version pins.
