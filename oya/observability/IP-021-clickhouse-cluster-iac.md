# IP-021 — Observability ClickHouse Cluster IaC

**Phase:** PHASE-02-OBSERVABILITY-CLICKHOUSE-EXTENSION (new phase introduced 2026-05-18; addendum at `microservices/observability/PHASE-02-OBSERVABILITY-CLICKHOUSE-EXTENSION-ADDENDUM.md`)
**Owner:** infra (axis-observability + ops-sre-reliability)
**Authority ADRs:** ADR-0193 OLAP analytics warehouse canonical, ADR-0186 observability backplane (Stage 2 "Storage"), ADR-0131 per-microservice flat layout, ADR-0145 inter-microservice communication, ADR-0184 storage tier layering
**Status:** Planned
**Phase trace:** PHASE-02 §"Cluster bootstrap — first work item" (addendum lines 14-20).

## Scope

Stand up the **observability µservice's ClickHouse cluster** (ClickHouse 26.3 LTS) for telemetry rollups + ops portal queries. This cluster is **distinct from** the analytics µservice's tenant-facing cluster (separately deployed under the analytics µservice). The split is intentional per ADR-0186 Stage 2 "Storage" — observability owns telemetry rollups; analytics owns tenant-facing business analytics; they never share a cluster.

Per ADR-0193 §"Cluster shape", this deployment is distributed via **ClickHouse Keeper** (Raft consensus replaces ZooKeeper). 6 server pods + 3 Keeper pods is the baseline shape for a medium cell.

## File targets

| Path | Action | Line range | Notes |
|---|---|---|---|
| `microservices/observability/iac/helm/clickhouse/Chart.yaml` | exists | 1-25 | infra |
| `microservices/observability/iac/helm/clickhouse/values.yaml` | exists | 1-130 | infra |
| `microservices/observability/iac/helm/clickhouse/templates/external-secret.yaml` | exists | 1-25 | infra |
| `microservices/observability/iac/helm/clickhouse/templates/keeper-template.yaml` | exists | 1-60 | infra |
| `microservices/observability/iac/helm/clickhouse/templates/prometheus-rule.yaml` | exists | 1-60 | infra |
| `microservices/observability/iac/helm/clickhouse/templates/service-monitor.yaml` | exists | 1-15 | infra |
| `microservices/observability/iac/helm/clickhouse/templates/server-statefulset.yaml.tpl` | reference | n/a — chart bundled | infra |
| `microservices/observability/iac/helm/clickhouse/templates/keeper-statefulset.yaml.tpl` | reference | n/a — chart bundled | infra |
| `microservices/observability/iac/kustomize/components/clickhouse-keeper/keeper-cluster.yaml` | exists | 1-80 | infra |
| `microservices/observability/iac/kustomize/components/clickhouse-keeper/keeper-rbac.yaml` | create | 1-50 | infra |
| `microservices/observability/iac/kustomize/overlays/pack-kr/clickhouse/kustomization.yaml` | create | 1-25 | infra+pack-kr |
| `microservices/observability/iac/kustomize/overlays/pack-eu/clickhouse/kustomization.yaml` | create | 1-25 | infra+pack-eu |
| `microservices/observability/iac/kustomize/components/observability-namespace/namespace.yaml` | create | 1-15 | infra |
| `microservices/observability/iac/kustomize/components/observability-namespace/rbac.yaml` | create | 1-50 | infra |
| `microservices/observability/tests/integration/clickhouse_cluster_smoke.rs` | create | 1-200 | backend |
| `microservices/observability/tests/integration/clickhouse_pack_overlay_kr.rs` | create | 1-110 | backend |
| `microservices/observability/tests/integration/clickhouse_pack_overlay_eu.rs` | create | 1-110 | backend |

## Cluster topology (per ADR-0193 §"Cluster shape")

| Plane | Components | Replica count | Notes |
|---|---|---|---|
| Storage / compute | ClickHouse server | 6 (3 shards × 2 replicas) | MergeTree engines |
| Coordination | ClickHouse Keeper | 3 (Raft quorum) | Replaces ZooKeeper |
| Query routing | Distributed-table proxy | embedded (no separate pod) | server pods host Distributed engine |
| External object | SeaweedFS S3-compat | per-cell | Cold tier; covered in IP-024 |

## Deliverables

1. **Helm chart** — already authored at `microservices/observability/iac/helm/clickhouse/`; this IP completes any remaining template and validates.
2. **Per-pack overlays** — KR + EU residency overlays pin the cell's SeaweedFS cold-tier endpoint + per-pack `database.namespace` prefix.
3. **PrometheusRule + ServiceMonitor** — already authored; this IP verifies scrape labels and alert thresholds match the ingest-throughput + query-latency SLOs.
4. **ClickHouse Keeper Kustomize component** — already authored at `iac/kustomize/components/clickhouse-keeper/`; this IP adds RBAC.
5. **Observability namespace bootstrap** — namespace creation, root-credential ExternalSecret, RBAC (observability-ch-admin / observability-ch-reader ServiceAccounts), NetworkPolicy (deny-all + allowlist for OTel collector + ops-portal).
6. **Cluster smoke test** — verifies all pods reach Ready and a sample query round-trip succeeds.

## Acceptance criteria

- All **6 server pods + 3 Keeper pods** reach Ready within 5min of `helm install`.
- ClickHouse Keeper Raft quorum reaches consensus within 30s of first pod ready.
- Distributed-table proxy on server pods routes queries to all 3 shards.
- OpenTelemetry Collector gateway (deployed separately) successfully writes to the cluster via the `clickhouseexporter` (validated by IP-022 smoke).
- Query latency **p99 ≤ 1s** for typical ops-portal rollups (validated against the canary MV at `system.mv_canary_health`).
- ServiceMonitor `up{job="clickhouse"}` is `1` within 60s of cluster ready.
- PrometheusRule loads without parse errors; alert thresholds match `clickhouse-ingest-throughput.openslo.yaml` + `query-latency-logs.openslo.yaml`.
- KR + EU pack overlays apply cleanly without manual edits; pack-eu overlay denies egress to non-eu cells.

## Test plan

| Test | Verifies |
|---|---|
| `test_helm_install_dev_cell` | helm install succeeds in dev cell |
| `test_all_server_pods_ready` | 6 server pods Ready |
| `test_keeper_quorum` | 3-pod Raft quorum |
| `test_keeper_raft_leader_election` | leader elected within 30s of cluster ready |
| `test_distributed_engine_routes_to_all_shards` | sample query distributes across 3 shards |
| `test_sample_table_roundtrip` | CREATE TABLE + INSERT + SELECT + DROP |
| `test_prometheus_rule_loaded` | rule body present |
| `test_service_monitor_scraped` | `up{job="clickhouse"}` = 1 |
| `test_pack_kr_overlay_applies` | KR overlay sets kr-* endpoint |
| `test_pack_eu_overlay_applies` | EU overlay sets eu-* endpoint |
| `test_namespace_rbac_least_privilege` | reader cannot DROP TABLE; admin can |
| `test_keeper_pod_loss_no_data_loss` | one Keeper pod lost → quorum holds; data ingestion continues |

## Evidence emission

- **Audit chain (per ADR-0145):** cluster-bootstrap event with `{cell_id, helm_revision, pod_inventory_hash, keeper_quorum_state, completed_at_ts}` emitted to `oya.observability.audit.clickhouse.bootstrap`; sealed via Ed25519.
- **Metrics:** ServiceMonitor scrape exposes `ClickHouseProfileEvents_Query`, `ClickHouseProfileEvents_InsertQuery`, `ClickHouseProfileEvents_DiskReadElapsedMicroseconds`, `ClickHouseKeeperRaftIsLeader`, replication lag.
- **Dashboard:** `microservices/observability/dashboards/clickhouse-overview.json` wired to per-cell Prometheus.
- **Evidence pack:** `evidence/observability-clickhouse-cluster-bootstrap-<cell-id>.json` on every helm-install run.

## Rollback procedure

1. **Detection.** `ClickHouseClusterRolloutFailed` alert fires when ≥ 1 server pod or ≥ 1 Keeper pod fails to reach Ready within 5min.
2. **Triage.** Page observability-oncall via PagerDuty + Opsgenie (dual-vendor per ADR-0186).
3. **Helm rollback.** `helm rollback observability-clickhouse <previous-revision> -n observability`. Wait 5min for converge.
4. **Verification.** Re-run smoke test against the rolled-back release.
5. **Last-resort destructive rollback.** `helm uninstall observability-clickhouse -n observability` then re-install prior revision via ArgoCD ApplicationSet. PVs retained (StorageClass `Retain`).
6. **Data preservation.** Storage PVs + SeaweedFS objects retained across uninstall.
7. **Post-incident.** File `evidence/incidents/clickhouse-rollback-<date>.json`.

## Blocking deps

- ADR-0193 promoted to Accepted (Accepted 2026-05-18).
- SeaweedFS S3-compat available in target cell (per Fix-S) for cold-tier storage (IP-024).
- Per-cell capacity-model (`microservices/observability/capacity-model-clickhouse.md`) sized for medium cell baseline.
- Observability namespace pre-created by the cell-bootstrap workflow.

## Exit criteria

All acceptance criteria pass in dev cell; smoke test green for 7 consecutive CI runs; audit-chain bootstrap event present; observability-oncall has read + drilled the runbook at `microservices/observability/runbooks/clickhouse.md`; PrometheusRule alerts have fired correctly for at least one synthetic failure (chaos drill).

## Out of scope

- OTel → ClickHouse bridge configuration (IP-022).
- Ops portal rollup materialized views (IP-023).
- Cold-tier retention policy (IP-024).
- Backup + restore drill (IP-025).
- Tenant-facing analytics cluster (analytics µservice).

## Capacity sizing baseline (medium cell)

| Resource | Server (per pod) | Keeper (per pod) |
|---|---|---|
| CPU request | 4 | 1 |
| CPU limit | 8 | 2 |
| Memory request | 16Gi | 2Gi |
| Memory limit | 32Gi | 4Gi |
| Disk (PV) | 500Gi (hot tier) | 20Gi (Raft log + snapshot) |
| Network bandwidth | 10Gbps recommended | 1Gbps |

Small cell: 3 server + 3 Keeper. Large cell: 12 server (6 shards × 2 replicas) + 5 Keeper. Per-cell sizing read from `microservices/observability/capacity-model-clickhouse.md`.

## Security posture

- **mTLS via the mesh.** Server-server traffic and server-Keeper traffic both via mesh mTLS.
- **TLS ingress.** Native ClickHouse `tcp_secure` (port 9440) for client connections; mesh certificate via cert-manager.
- **AuthN.** Server-level users (writer/reader/admin) bootstrapped at install via ExternalSecret-managed `users.xml`. Per-tenant database access enforced by row-level policies (per ADR-0193 §"Multi-tenancy isolation") applied in IP-023.
- **AuthZ via Cedar.** Pre-query Cedar check at the ops-portal middleware; row-level policy is the second-line defense.
- **Network policy.** Default deny; explicit ingress allowlist for OTel Collector gateway + ops-portal + DDL bootstrap Job + monitoring.
- **Audit chain.** Every bootstrap action emits a signed audit event.
- **Secret rotation.** Server passwords rotated every 90 days via ExternalSecret.

## References

- ADR-0193 — OLAP analytics warehouse canonical.
- ADR-0186 — observability backplane.
- ADR-0131 — per-microservice flat layout.
- ADR-0145 — communication reform.
- ADR-0184 — storage tier layering.
- Runbooks: `microservices/observability/runbooks/clickhouse.md`, `clickhouse-restore.md`.
- Capacity model: `microservices/observability/capacity-model-clickhouse.md`.
- OpenSLOs: `clickhouse-ingest-throughput.openslo.yaml`, `query-latency-logs.openslo.yaml`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/observability/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/observability/IP-021-clickhouse-cluster-iac.md` matched `p99, SLO`; anchors `microservices/observability/runbooks/clickhouse-restore.md, crates/oya-cloud-observability-api/src/lib.rs`; type anchor `crates/oya-cloud-observability-api/src/lib.rs::CloudObservabilityAuditRecord`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/observability/IP-021-clickhouse-cluster-iac.md` matched `emission`; anchors `microservices/observability/manifest.json, crates/oya-cloud-observability-api/src/lib.rs`; type anchor `crates/oya-cloud-observability-api/src/lib.rs::CloudObservabilityAuditRecord`.
