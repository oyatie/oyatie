# IP-091 — Foundry Milvus Cluster IaC

**Phase:** PHASE-02-FOUNDRY-DATA-SUBSTRATE (new phase introduced 2026-05-18; addendum at `microservices/intelligence/PHASE-02-FOUNDRY-DATA-SUBSTRATE-ADDENDUM.md`)
**Owner:** infra (axis-foundry + ops-sre-reliability)
**Authority ADRs:** ADR-0192, ADR-0131 per-microservice flat layout, ADR-0184 storage tier layering, ADR-0136 foundry-as-single-microservice, ADR-0145 inter-microservice communication reform
**Status:** Planned
**Phase trace:** PHASE-02 §"Data substrate bootstrap — first work item" (addendum line 18-24).

## Scope

Stand up the Foundry-owned **Milvus 2.6.x disaggregated cluster** per ADR-0192 §"Cluster shape — disaggregated, four-plane". This IP delivers the cluster-grade IaC: Helm chart, per-pack overlays, observability wiring, and namespace bootstrap. All seven downstream Milvus IPs (092..097) depend on this IP reaching `Accepted`.

The cluster is foundry-namespace; the canonical Helm chart lives at `microservices/intelligence/iac/helm/milvus/` and is already partially authored. This IP completes the chart, verifies all four planes (Access / Coordinator / Worker / Storage) reach steady state, and exercises a smoke round-trip.

## File targets (paths + line ranges)

| Path | Action | Line range | Owner |
|---|---|---|---|
| `microservices/intelligence/iac/helm/milvus/Chart.yaml` | exists | 1-30 | infra |
| `microservices/intelligence/iac/helm/milvus/values.yaml` | exists | 1-150 | infra |
| `microservices/intelligence/iac/helm/milvus/templates/external-secret.yaml` | exists | 1-30 | infra |
| `microservices/intelligence/iac/helm/milvus/templates/network-policy.yaml` | exists | 1-40 | infra |
| `microservices/intelligence/iac/helm/milvus/templates/prometheus-rule.yaml` | exists | 1-70 | infra |
| `microservices/intelligence/iac/helm/milvus/templates/service-monitor.yaml` | exists | 1-30 | infra |
| `microservices/intelligence/iac/helm/milvus/templates/coord-statefulset.yaml.tpl` | reference | n/a — chart bundled | infra |
| `microservices/intelligence/iac/helm/milvus/templates/worker-deployment.yaml.tpl` | reference | n/a — chart bundled | infra |
| `microservices/intelligence/iac/kustomize/overlays/pack-kr/milvus/kustomization.yaml` | create | 1-25 | infra+pack-kr |
| `microservices/intelligence/iac/kustomize/overlays/pack-eu/milvus/kustomization.yaml` | create | 1-25 | infra+pack-eu |
| `microservices/intelligence/iac/kustomize/components/milvus-foundry-namespace/namespace.yaml` | create | 1-20 | infra |
| `microservices/intelligence/iac/kustomize/components/milvus-foundry-namespace/rbac.yaml` | create | 1-60 | infra |
| `microservices/intelligence/tests/integration/milvus_cluster_smoke.rs` | create | 1-200 | backend |

## Cluster topology (per ADR-0192 §"Cluster shape")

| Plane | Components | Replica count | Notes |
|---|---|---|---|
| Access | Proxy nodes | 3 (HA) | gRPC ingress; Cedar+OIDC middleware in front |
| Coordinator | Root / Query / Data / Index coord | 2 each (active+passive) | Etcd-elected; passive ready for failover |
| Worker | Query / Data / Index nodes | 6 / 4 / 2 | Sized for medium cell baseline |
| Storage — meta | Etcd | 3 (Raft quorum) | Embedded with chart |
| Storage — message | Pulsar | 3 brokers + 3 bookies | Embedded with chart |
| Storage — object | SeaweedFS S3-compat | Pre-provisioned per-cell | External endpoint |

## Deliverables

1. **Helm chart** — already authored at `microservices/intelligence/iac/helm/milvus/`; this IP audits + completes any missing template.
2. **Per-pack overlays** — KR + EU residency overlays at `iac/kustomize/overlays/pack-{kr,eu}/milvus/`. Each overlay pins the cell's SeaweedFS endpoint, sets the per-pack `database.namespace` prefix, and applies pack-specific network policies.
3. **ServiceMonitor + PrometheusRule** — already authored; verify scrape labels match `microservice=foundry, substrate=milvus`.
4. **Cluster smoke test** — Rust integration test at `microservices/intelligence/tests/integration/milvus_cluster_smoke.rs` verifying 4-plane health (see Test plan below).
5. **Foundry namespace bootstrap** — namespace creation, root-credential ExternalSecret, RBAC (foundry-milvus-admin / foundry-milvus-reader ServiceAccounts), NetworkPolicy (deny-all + allowlist).

## Test plan (test names + acceptance criteria)

| Test | Location | Verifies |
|---|---|---|
| `test_helm_install_dev_cell` | `tests/integration/milvus_cluster_smoke.rs` | `helm install foundry-milvus microservices/intelligence/iac/helm/milvus/` succeeds in dev cell |
| `test_all_coord_pods_ready` | same | all 4 coord types reach Ready (active + passive) |
| `test_all_worker_pods_ready` | same | 6 query + 4 data + 2 index pods Ready |
| `test_etcd_quorum` | same | etcdctl endpoint status reports 3-node Raft quorum |
| `test_pulsar_brokers_ready` | same | 3-broker Pulsar cluster has live brokers |
| `test_seaweedfs_s3_reachable` | same | S3 `ListBuckets` succeeds from inside the cluster |
| `test_sample_collection_roundtrip` | same | create collection → insert 100 vectors → search top-10 → drop |
| `test_prometheus_rule_loaded` | same | `kubectl get prometheusrules foundry-milvus -n foundry` returns the rule body |
| `test_service_monitor_scraped` | same | `up{job="milvus"}` is `1` after 60s |
| `test_pack_kr_overlay_applies` | `tests/integration/milvus_pack_overlay_kr.rs` | KR overlay sets the kr-* S3 endpoint and survives `kubectl diff` against base |
| `test_pack_eu_overlay_applies` | `tests/integration/milvus_pack_overlay_eu.rs` | EU overlay sets the eu-* S3 endpoint |
| `test_namespace_rbac_least_privilege` | `tests/integration/milvus_rbac.rs` | foundry-milvus-reader cannot delete; foundry-milvus-admin can |

## Acceptance criteria

- `helm install foundry-milvus microservices/intelligence/iac/helm/milvus/` succeeds in dev cell within 5min.
- All coordinator pods (active + passive per coord type, 8 pods total) reach Ready within 3min.
- All worker pods (6 query + 4 data + 2 index, 12 pods total) reach Ready within 3min.
- Etcd 3-node quorum reaches consensus within 30s of first pod ready.
- Pulsar 3-broker cluster reaches Ready within 90s.
- SeaweedFS S3-compat external endpoint reachable (configured per-cell, validated by the smoke test).
- Sample-collection round-trip (create + insert 100 vectors + search top-10 + drop) succeeds end-to-end.
- ServiceMonitor `up{job="milvus"}` is `1` within 60s of cluster ready.
- PrometheusRule loads without parse errors.
- KR + EU pack overlays apply cleanly without manual edits.

## Evidence emission

- **Audit chain (per ADR-0145):** cluster-bootstrap event with `{cell_id, helm_revision, pod_inventory_hash, completed_at_ts}` emitted to `oya.foundry.audit.milvus.bootstrap` Pulsar topic; sealed via Ed25519 by the foundry audit emitter.
- **Metrics:** ServiceMonitor scrape exposes `milvus_proxy_req_count`, `milvus_proxy_req_latency_bucket`, `milvus_storage_segment_count`, `milvus_index_node_idle_ratio`.
- **Dashboard:** `microservices/intelligence/dashboards/milvus-overview.json` (existing) wired to the per-cell Prometheus.
- **Evidence pack:** `evidence/foundry-milvus-cluster-bootstrap-<cell-id>.json` emitted to `microservices/intelligence/evidence/` on every successful helm-install run.

## Rollback procedure

1. **Detection.** `MilvusClusterRolloutFailed` alert fires when ≥1 coordinator or ≥3 worker pods fail to reach Ready within 5min.
2. **Triage.** Page foundry-oncall via PagerDuty + Opsgenie (dual-vendor per ADR-0186).
3. **Helm rollback.** `helm rollback foundry-milvus <previous-revision> -n foundry`. Wait 3min for converge.
4. **Verification.** Re-run smoke test against the rolled-back release.
5. **Last-resort destructive rollback.** If `helm rollback` cannot stabilise, `helm uninstall foundry-milvus -n foundry` then re-install the prior revision via the per-cell ArgoCD ApplicationSet (gitops drift will reconcile within 5min).
6. **Data preservation.** Etcd PVs and SeaweedFS objects are retained across uninstall (StorageClass `Retain` policy). Re-install reuses prior state.
7. **Post-incident.** Author `evidence/incidents/milvus-rollback-<date>.json` with the failure mode + remediation.

## Blocking deps

- ADR-0192 promoted to Accepted (already Accepted 2026-05-18).
- SeaweedFS S3-compat available in the target cell (per Fix-S).
- Per-cell capacity-model (`microservices/intelligence/capacity-model-milvus.md`) sized for medium cell baseline.
- Foundry namespace pre-created by the cell-bootstrap workflow.

## Exit criteria

All acceptance criteria pass in dev cell; cluster smoke test green in CI for 7 consecutive runs; audit-chain bootstrap event present in `oya.foundry.audit.milvus.bootstrap`; foundry-oncall has read + drilled the runbook at `microservices/intelligence/runbooks/milvus.md`. Once exit criteria are satisfied, IP-091 transitions to `Accepted` and IP-092..IP-097 unblock.

## Out of scope

- Per-tenant collection bootstrap (IP-092).
- Embedding ingest pipeline (IP-093).
- HNSW tuning + adapter crate (IP-094).
- GPU acceleration (IP-095 — optional per-cell).
- Backup + restore drill (IP-096).
- Cross-region replication (IP-097).
- Cell capacity-planning workflow (separate IP under the capacity µservice).

## Capacity sizing baseline (medium cell)

| Resource | Coord (per pod) | Proxy (per pod) | Query (per pod) | Data (per pod) | Index (per pod) |
|---|---|---|---|---|---|
| CPU request | 1 | 2 | 4 | 2 | 4 |
| CPU limit | 2 | 4 | 8 | 4 | 8 |
| Memory request | 2Gi | 4Gi | 16Gi | 8Gi | 16Gi |
| Memory limit | 4Gi | 8Gi | 32Gi | 16Gi | 32Gi |
| Disk (PV) | 10Gi | n/a | 100Gi (cache) | 200Gi | 100Gi (build) |

Small cell scales the worker plane to 3 query + 2 data + 1 index. Large cell scales to 12 query + 8 data + 4 index. Per-cell sizing is read from `microservices/intelligence/capacity-model-milvus.md`.

## Security posture

- **mTLS everywhere.** Proxy ingress requires SPIFFE workload identity. Internal coord-worker traffic is mTLS via the service mesh.
- **AuthN at proxy.** Milvus built-in user auth is bootstrapped at install with a strong root password (ExternalSecret-managed); per-µservice / per-tenant credentials are separate.
- **AuthZ via Cedar.** Foundry-providers-rest middleware short-circuits any cross-tenant request before it reaches the proxy (defense in depth).
- **Network policy.** Default deny; explicit ingress allowlist for foundry-providers-rest + foundry-milvus-ingest-app + foundry-milvus-tenant-bootstrap + monitoring.
- **Audit chain.** Every bootstrap action emits a signed audit event (Ed25519).
- **Secret rotation.** Root password rotated via the ExternalSecret operator on a 90-day schedule.

## References

- ADR-0192 — vector database canonical Milvus.
- ADR-0184 — storage tier layering.
- ADR-0136 — foundry-as-single-microservice.
- ADR-0131 — per-microservice flat layout.
- ADR-0145 — inter-microservice communication reform (audit chain emission).
- Runbook: `microservices/intelligence/runbooks/milvus.md`.
- Capacity model: `microservices/intelligence/capacity-model-milvus.md`.
- OpenSLOs: `microservices/intelligence/slos/milvus-search-latency.openslo.yaml`, `milvus-ingest-lag.openslo.yaml`.

## Wave 15 counterpart anchor

- Counterparts: Snowflake Cortex Search, Databricks Vector Search, OpenAI vector stores, and Palantir AIP ontology retrieval.
- Gap closure: this IP closes Foundry retrieval/vector substrate for tenant-isolated agent grounding and eval replay.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
