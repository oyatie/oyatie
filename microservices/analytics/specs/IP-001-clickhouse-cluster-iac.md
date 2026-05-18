# IP-001 — Analytics ClickHouse Cluster IaC

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** infra (council-analytics + ops-sre-reliability)
**Authority ADRs:** ADR-0193, ADR-0184, ADR-0131-per-microservice-flat-layout, ADR-0043 secrets, ADR-0148 NetworkPolicy
**Status:** Planned

## Scope

Stand up the analytics µservice's ClickHouse 26.3 LTS cluster in dev cell + KR cell, using the canonical Helm chart at `microservices/analytics/iac/helm/clickhouse-analytics/` (per ADR-0131 flat layout). The chart consumes the upstream Altinity clickhouse-operator + a `ClickHouseInstallation` CR shaping the cluster into 3 shards × 2 replicas + a separate 3-node ClickHouse Keeper Raft quorum.

This IP delivers the cluster substrate that every subsequent IP builds on. It does NOT cover:

- Per-tenant database bootstrap (IP-002).
- Adapter-crate scaffold (IP-003).
- Ingest pipeline (IP-004).

## Deliverables

1. Helm chart `microservices/analytics/iac/helm/clickhouse-analytics/` (already authored as part of IaC scaffolding).
2. Per-pack overlays at `microservices/analytics/iac/kustomize/overlays/pack-{kr,eu,ksa,uae,us-healthcare}/` (already authored).
3. Flux Kustomization at `microservices/analytics/iac/kustomize/base/kustomization.yaml`.
4. OpenBao External Secret for admin password + S3 access keys.
5. PrometheusRule for cluster-health alerts (Keeper quorum, replication lag, partition merge backlog, instance down) — already in Helm template.
6. NetworkPolicy restricting ClickHouse access to the analytics namespace + the observability namespace (scrape).
7. ServiceMonitor for Prometheus scrape (per ADR-0186 Stage 1+2).
8. Pre-deployment smoke test (`scripts/iac/clickhouse-smoke-test.sh`) verifying TCP/HTTP reachability + Keeper leader presence + sample query round-trip.

## Acceptance criteria

- `helm install analytics-clickhouse microservices/analytics/iac/helm/clickhouse-analytics/` succeeds in dev cell.
- All 6 server pods + 3 Keeper pods reach `Ready=True` within 5 minutes.
- Keeper has a leader: `clickhouse_keeper_is_leader == 1` on exactly one pod.
- Sample query `SELECT version()` returns `26.3.10.60` on all 6 server replicas.
- ServiceMonitor surfaces `ClickHouseProfileEvents_Query` metrics in Prometheus.
- NetworkPolicy denies traffic from a probe pod in an unrelated namespace.
- OpenBao-resolved secret `clickhouse-admin` is present in the namespace.
- KR overlay produces an identical cluster topology in the KR cell with `kr-seoul`-bound S3 endpoint.
- EU overlay produces same in eu-frankfurt with eu-bound S3.
- Smoke test exits 0.

## Implementation tasks

### T1 — Helm chart skeleton

Files already authored in IaC scaffolding:

- `microservices/analytics/iac/helm/clickhouse-analytics/Chart.yaml`
- `microservices/analytics/iac/helm/clickhouse-analytics/values.yaml`
- `microservices/analytics/iac/helm/clickhouse-analytics/templates/clickhouse-installation.yaml`
- `microservices/analytics/iac/helm/clickhouse-analytics/templates/external-secret.yaml`
- `microservices/analytics/iac/helm/clickhouse-analytics/templates/service-monitor.yaml`
- `microservices/analytics/iac/helm/clickhouse-analytics/templates/prometheus-rule.yaml`
- `microservices/analytics/iac/helm/clickhouse-analytics/templates/network-policy.yaml`

The shape mirrors the observability sibling chart (`microservices/observability/iac/helm/clickhouse/`) but the namespace, OpenBao secret path, and tenant-facing intent differ. The analytics µservice's ClickHouse cluster is sized for tenant query QPS (more replicas in the search path); the observability sibling is sized for ingest throughput.

Key shape:

- 3 shards × 2 replicas (`server.shards=3`, `server.replicasPerShard=2`).
- 3-node Keeper quorum (`keeper.replicas=3`).
- Pod anti-affinity by hostname (replicas on distinct nodes).
- `oya.disk-class: nvme-premium` node selector.
- Storage policy `hot_cold` with `s3_cold` disk.

### T2 — Per-pack overlays

Already authored at:

- `microservices/analytics/iac/kustomize/overlays/pack-kr/` — Korean pack.
- `microservices/analytics/iac/kustomize/overlays/pack-eu/` — European pack.
- `microservices/analytics/iac/kustomize/overlays/pack-ksa/` — Kingdom of Saudi Arabia.
- `microservices/analytics/iac/kustomize/overlays/pack-uae/` — UAE.
- `microservices/analytics/iac/kustomize/overlays/pack-us-healthcare/` — HIPAA-attested.

Each overlay:

- Patches `cold-tier-patch.yaml` with the regional S3 endpoint.
- Patches `node-selector-patch.yaml` with `oya.pack=<region>`.
- (pack-us-healthcare adds `hipaa-audit-patch.yaml`.)

### T3 — Smoke test

File: `scripts/iac/clickhouse-smoke-test.sh`

```bash
#!/bin/bash
set -euo pipefail

NS=${NAMESPACE:-analytics}
CLUSTER=${CLUSTER:-analytics-clickhouse-1}

echo "[1/5] Keeper pods Ready..."
KEEPER_READY=$(kubectl get pods -n "$NS" -l app.kubernetes.io/component=clickhouse-keeper -o json | jq '[.items[] | select(.status.conditions[]?.type=="Ready" and .status.conditions[]?.status=="True")] | length')
if [ "$KEEPER_READY" -lt 3 ]; then
    echo "FAIL: $KEEPER_READY/3 Keepers Ready"
    exit 1
fi

echo "[2/5] Server pods Ready..."
SERVER_READY=$(kubectl get pods -n "$NS" -l app.kubernetes.io/component=clickhouse-server -o json | jq '[.items[] | select(.status.conditions[]?.type=="Ready" and .status.conditions[]?.status=="True")] | length')
if [ "$SERVER_READY" -lt 6 ]; then
    echo "FAIL: $SERVER_READY/6 Servers Ready"
    exit 1
fi

echo "[3/5] Keeper has a leader..."
LEADER_COUNT=0
for pod in $(kubectl get pods -n "$NS" -l app.kubernetes.io/component=clickhouse-keeper -o name); do
    MODE=$(kubectl exec -n "$NS" "$pod" -- clickhouse-keeper-client -p 9181 -q "stat" 2>/dev/null | grep -i "Mode:" | awk '{print $2}' || echo "")
    if [ "$MODE" = "leader" ]; then
        LEADER_COUNT=$((LEADER_COUNT + 1))
    fi
done
if [ "$LEADER_COUNT" -ne 1 ]; then
    echo "FAIL: Expected exactly 1 leader, got $LEADER_COUNT"
    exit 1
fi

echo "[4/5] Sample query round-trip..."
VERSION=$(kubectl exec -n "$NS" deployment/clickhouse-server-0 -- clickhouse-client --query "SELECT version()" 2>/dev/null || true)
if [[ ! "$VERSION" =~ ^26\.3\. ]]; then
    echo "FAIL: Expected version 26.3.x, got $VERSION"
    exit 1
fi

echo "[5/5] NetworkPolicy denies external pod..."
kubectl run smoke-probe --image=curlimages/curl --restart=Never -n default -- curl -s --max-time 5 http://clickhouse.analytics.svc.cluster.local:8123/ping
PROBE_EXIT=$?
kubectl delete pod smoke-probe -n default --ignore-not-found
if [ "$PROBE_EXIT" -eq 0 ]; then
    echo "FAIL: NetworkPolicy did not deny external probe"
    exit 1
fi

echo "ALL SMOKE TESTS PASSED"
```

### T4 — PrometheusRule wiring

Already in the Helm template (`prometheus-rule.yaml`). Rules emit to AlertManager → PagerDuty + Opsgenie via the canonical webhook (per ADR-0186 Stage 4):

- `ClickHouseKeeperNoLeader` (`severity: page`; runbook `keeper-quorum-recovery.md`).
- `ClickHouseReplicationLag > 60s` (`severity: page`; runbook `clickhouse.md`).
- `ClickHousePartMergeBacklog > 100` (`severity: ticket`; runbook `ingest-lag-burn.md`).
- `ClickHouseInstanceDown` (`severity: page`; runbook `clickhouse.md`).

Additional rules added in IP-006 (cold-tier):

- `ClickHouseColdTierS3ErrorRate > 5%`.
- `ClickHouseColdTierQueryLatency > 2s p99`.

### T5 — Flux Kustomization wire-up

The base `kustomization.yaml` (already authored) renders the Helm chart with defaults. Each cell-local Flux GitOps repository references a per-pack overlay:

```yaml
# Flux config in `clusters/dev-cell/kustomization.yaml`
resources:
  - ../../../microservices/analytics/iac/kustomize/base
```

```yaml
# Flux config in `clusters/kr-seoul-1/kustomization.yaml`
resources:
  - ../../../microservices/analytics/iac/kustomize/overlays/pack-kr
```

### T6 — Integration test (deployment)

File: `microservices/analytics/iac/helm/clickhouse-analytics/tests/test-deploy.sh`

```bash
#!/bin/bash
set -euo pipefail

helm install analytics-clickhouse-test microservices/analytics/iac/helm/clickhouse-analytics/ \
    --namespace analytics-test \
    --create-namespace \
    --wait \
    --timeout 5m \
    --set keeper.replicas=3 \
    --set server.replicas=6

scripts/iac/clickhouse-smoke-test.sh

helm uninstall analytics-clickhouse-test --namespace analytics-test
```

## Out of scope

- Per-tenant database creation (IP-002).
- ClickHouse user provisioning beyond `admin` (IP-002).
- TTL / cold-tier policy at the table level (IP-006 — storage config is in this IP; per-table TTL is in IP-006).
- Ingest pipeline (IP-004).

## Capacity model

Per-cell production sizing (analytics µservice):

- 6 server nodes × 4 vCPU / 16 GiB / 500GiB NVMe.
- 3 Keeper nodes × 0.5 vCPU / 1 GiB / 20GiB NVMe.
- S3 cold tier auto-scales.
- Network: 10 Gbps inter-pod; 1 Gbps inter-cell (federation path).

(See `microservices/analytics/capacity-model.md` for full capacity ceilings.)

## Failure modes

| Mode | Detection | Mitigation |
|---|---|---|
| Pod OOMKilled | Kubernetes events | Memory limit tuning; `runbooks/clickhouse.md` |
| Keeper quorum loss | `ClickHouseKeeperNoLeader` alert | `runbooks/keeper-quorum-recovery.md` |
| Helm install hangs | `helm install --wait` timeout | rollback; investigate; alert |
| PVC provisioning slow | pod stays Pending | StorageClass diagnostic |
| OpenBao secret unavailable | external-secret reports Failed | OpenBao health check |

## SLO commitment (downstream IP-014)

- Pod-level: 99.9% pod availability per replica.
- Cluster: 99.95% cluster availability (3 shards × 2 replicas; tolerate single-replica failure per shard) — per `slos/cluster-availability.openslo.yaml`.
- Keeper: 99.99% quorum availability (3-node Raft) — per `slos/keeper-quorum-availability.openslo.yaml`.

## Rollback

- Helm rollback: `helm rollback analytics-clickhouse <revision>`.
- For full retreat: `helm uninstall analytics-clickhouse` (DROPs all data; use only in dev).

## Evidence emission

- Per smoke test run: `evidence/smoke-tests/clickhouse-cluster-<cell>-<date>.json`.
- Per Helm install: `evidence/helm/installs/analytics-clickhouse-<cell>-<date>.json`.
- Cluster health: continuous Prometheus scrape.

## Runbook hooks

`microservices/analytics/runbooks/clickhouse.md` (general); `keeper-quorum-recovery.md`, `ingest-lag-burn.md`, `cold-tier-latency.md`, `restore-drill.md`, `capacity-rebalance.md`, `mv-lag-triage.md`, `tenant-onboard-failure.md` — all 8 authored.

## References

- ADR-0193 §"Cluster shape — coordinator-free via ClickHouse Keeper".
- ADR-0184 §"Tier boundary rules".
- ADR-0131-per-microservice-flat-layout.
- ADR-0043-secrets-management-openbao-and-hsm-per-cell (External Secret pattern).
- ADR-0148 (NetworkPolicy + service mesh interaction).
- ADR-0186 Stages 1–5 (observability wiring).
- Altinity clickhouse-operator: https://github.com/Altinity/clickhouse-operator.
