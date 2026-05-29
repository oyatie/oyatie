# Milvus Runbook — Foundry

**Authority:** ADR-0192
**Owner:** axis-foundry + ops-sre-reliability
**Last reviewed:** 2026-05-18

## Cluster health quick-check

```bash
# All pods Ready
kubectl get pods -n foundry -l app.kubernetes.io/name=milvus

# Etcd quorum
kubectl exec -n foundry milvus-etcd-0 -- etcdctl endpoint status --cluster

# Pulsar broker availability
kubectl exec -n foundry pulsar-broker-0 -- bin/pulsar-admin brokers list public

# Sample query round-trip via the proxy
kubectl exec -n foundry milvus-proxy-0 -- /milvus/bin/milvus_cli health
```

## Coordinator failover

Active/passive per coord type (root/query/data/index). On active failure:

1. Detect via `MilvusCoordinatorDown` alert (auto-page).
2. Verify passive promoted: `kubectl logs milvus-rootcoord-passive -n foundry | grep "promoted"`.
3. If promotion did not occur within 60s, manually trigger via deleting the active pod.
4. After recovery, verify both pods reach steady state (one active, one passive).

## Search-latency burn rate

Alert `MilvusSearchLatencyBurnRate_FastBurn` (p99 > 30ms over 1h):

1. Identify hot collection(s): `SELECT collection_name, count() FROM milvus_proxy_req_latency WHERE le="0.030" GROUP BY collection_name ORDER BY count() DESC LIMIT 10`.
2. Inspect per-collection HNSW ef_search; tune up if recall is acceptable. (M / ef_construction require index rebuild.)
3. If hot collection has >100M vectors and HNSW is not adequate, consider migrating to DiskANN (cold tier) for the older partitions.
4. Capacity check: query-node CPU saturation? → scale `queryNode.replicas`.

## Ingest backlog (sealed segment count > 5K)

1. Index-node CPU bottleneck? Scale `indexNode.replicas`.
2. GPU enabled? If cell's ingest peak is consistently high and CPU is bottlenecked, consider opting into GPU acceleration per IP-095.

## Per-tenant DSR cascade

```bash
# Tenant offboard — drops all per-tenant collections + emits proof-of-erasure.
# This is automated by the tenant-bootstrap controller (IP-092). Manual override:
kubectl exec -n foundry milvus-proxy-0 -- /milvus/bin/milvus_cli drop tenant ten_acme --confirm
```

## Backup restore drill

See `microservices/intelligence/runbooks/milvus-restore.md`.

## Escalation

- Page → PagerDuty `foundry-oncall`.
- Page → Opsgenie `foundry-oncall` (dual-vendor per ADR-0186).
- Slack: `#foundry-incidents`.
