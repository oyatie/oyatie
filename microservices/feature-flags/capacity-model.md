---
doc_class: CapacityModel
microservice: feature-flags
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0248
  - ADR-0252
  - ADR-0254
companion_docs:
  - microservices/feature-flags/ARCHITECTURE.md
  - microservices/feature-flags/multi-region.md
  - microservices/feature-flags/slos/flag-eval-latency.openslo.yaml
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Capacity Model — Feature Flags

## Design load parameters

| Parameter | Value | Source |
|---|---|---|
| Active tenants at GA | 100,000 | PRD target |
| Peak concurrent SDK sessions | 10,000,000 | 100k tenants × 100 concurrent clients avg |
| Flag evaluation requests per second (peak) | 10,000,000 rps | PRD §NFR: ≥100k eval/s per replica × expected replicas |
| Flag definitions per tenant (p99) | 10,000 | LaunchDarkly reported 5k average; 2× safety margin |
| Flag mutations per minute per tenant (peak) | 60 | Rate-limit ceiling |
| Experiments active concurrently per tenant (p99) | 50 | Statsig reported 30 avg active experiments |
| Kill-switch activations per hour (platform-wide) | 10 | Incident cadence estimate; SRE capacity |

## Flag evaluation throughput

Applying **Little's Law**: `L = λ × W` where L = concurrent requests in-flight, λ = arrival rate, W = service time.

```
λ = 10,000,000 rps (peak platform-wide)
W = 1ms p99 (cell-local evaluation target)
L = 10,000,000 × 0.001 = 10,000 concurrent in-flight requests

Per replica capacity (Rust async, Tokio):
- Single replica: 100,000 rps at ≤1ms p99 (benchmarked; Axum + in-process Cedar Wasm)
- Required replicas at peak: 10,000,000 / 100,000 = 100 replicas platform-wide
- Per cell (50 Tier-2 cells): 100 / 50 = 2 replicas minimum; 4 replicas for N+1 headroom
- HPA target: 70% CPU utilization; scale out at 70,000 rps per replica
```

**Cache hit rate target:** ≥99% at client SDK (30s TTL). Cache hit eliminates 99% of network calls:
```
Effective server-side rps = 10,000,000 × (1 - 0.99) = 100,000 rps
Required replicas with ≥99% SDK cache: 100,000 / 100,000 = 1 replica min; 4 replicas for N+1
```

Bottleneck identification: SDK cache miss rate. If cache hit drops to 95%, server-side rps = 500,000 (5× growth). HPA covers this; at 5×: 20 replicas per cell. The platform cache strategy is the primary scale knob.

## Storage capacity

### Flag definitions (Postgres + Citus)

```
Per tenant: 10,000 flags × avg 2KB definition JSONB = 20MB per tenant
Platform: 100,000 tenants × 20MB = 2TB total flag-definition storage
Citus shards: 256 shards (tenant_id hash); avg shard size: 2TB / 256 = 7.8GB
Per Postgres replica: 256 / N_replicas shards; at 8 nodes: 32 shards × 7.8GB = 250GB per node
WAL replication: 3× replicas (1 primary + 2 standbys); storage: 250GB × 3 = 750GB per node group
```

### Experiment metric attribution (ClickHouse)

```
Metric events per day: 10,000,000 rps × 86,400s × 1% conversion-event rate = 8.6 billion events/day
ClickHouse row size: ~100 bytes (UUID + tenant + experiment + variant + metric + timestamp)
Daily ingest: 8.6B × 100B = 860GB/day
Retention: 90 days hot + 2 years cold (Parquet on object store)
Hot tier (90 days): 860GB × 90 = 77TB
Cold tier (2 years): 860GB × 730 × 0.1 (Parquet compression factor) = 62TB on object store
```

### Audit-chain events (sealed events)

```
Audit events per day: flag mutations (rare) + kill-switch + audit_required evaluations
Conservative estimate: 1M audit events/day × 512B per sealed event = 512MB/day
7-year retention: 512MB × 365 × 7 = 1.3TB (negligible vs metric attribution)
```

## Network capacity

Kill-switch activation fan-out (Kafka broadcast to 50 cells):
```
Message size: ~4KB (flag key + kill-switch state + signed payload)
Kafka partition throughput: 100MB/s per partition
Fan-out to 50 cells: 50 × 4KB × 10 kill-switches/hr = 2MB/hr (negligible)
```

Flag definition replication (Patroni WAL streaming):
```
Write rate: 60 mutations/min × 100k active tenants × 0.01% active concurrently = 60 mutations/s peak
WAL size per mutation: ~8KB
Replication bandwidth: 60 × 8KB = 480KB/s per region pair (negligible)
```

## Scaling thresholds

| Metric | Yellow (warning) | Red (page) | Auto-action |
|---|---|---|---|
| SDK cache miss rate | >5% | >10% | Alert; review TTL config |
| Eval p99 latency | >0.8ms | >1ms | HPA scale-out trigger |
| Flag-eval queue depth | >1,000 | >5,000 | HPA scale-out |
| Kill-switch fan-out latency | >500ms | >1s | Page SRE; failover to backup path |
| Postgres WAL lag | >1s | >5s | Alert; check replication health |
| ClickHouse ingest lag | >60s | >300s | Alert; check ingest pipeline |

## Multi-region capacity allocation

Per ADR-0248 cellular architecture:

| Cell | Tenant segment | Replica count | Storage allocation |
|---|---|---|---|
| us-cell-1 (primary) | US commercial tenants | 4 eval replicas; 1 Postgres primary + 2 standbys | 500GB Postgres; 30TB ClickHouse |
| eu-cell-1 (EU sovereign) | EU tenants (GDPR data-residency) | 4 eval replicas; 1+2 Postgres | 500GB; 30TB |
| kr-cell-1 (KR sovereign) | KR tenants (KR-ISMS-P) | 2 eval replicas; 1+1 Postgres | 200GB; 10TB |
| us-gov-cell-1 (FedRAMP) | FedRAMP tenants | 2 eval replicas; 1+1 Postgres | 200GB; 10TB |
| DR-pair cells | Failover for each primary | Same as primary (active-passive) | Same allocation |

## Capacity ceiling and horizontal scale-out

The system is horizontally scalable at all layers:
- Eval replicas: stateless; scale by adding Kubernetes pods.
- Postgres (Citus): add shards by increasing `shard_count`; rebalance operation is online.
- ClickHouse: add nodes to ClickHouse cluster; automatic shard rebalancing.
- Kafka: add partitions; kill-switch fan-out scales linearly.

**The system goes red when:**
1. SDK cache miss rate exceeds 20%: server-side rps = 2M; requires 20 eval replicas per cell.
2. ClickHouse ingest lag exceeds 5 minutes: metric attribution delayed; experiment results stale.
3. Postgres WAL lag exceeds 5 seconds: flag definition replication stale; cross-region consistency degraded.
