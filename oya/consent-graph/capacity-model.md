# consent-graph capacity model

- Owner: axis-consent-graph + sre-axis
- Date: 2026-05-18
- Authority: ADR-0214 §NFR scale targets.

## 1. Year-1 demand model

| Dimension | Year-1 peak | Year-3 projected |
|-----------|-------------|-------------------|
| Active agreements | 10M | 100M |
| New agreements / day | 100K | 1M |
| Revocations / day | 1M | 10M |
| Cross-tenant projection events / day | 100B | 1T |
| Enforcement evaluations / s peak | 100K | 1M |
| Concurrent partner-directory peers | 10K | 100K |
| Bilateral audit events / day | 50B | 500B |

## 2. Per-service capacity

### 2.1 consent-graph-app (agreement + enforcement + projection-gateway co-resident)

| Metric | Per-pod | 10 pods/region | 11 regions |
|--------|---------|----------------|------------|
| RPS sustained | 5K | 50K | 550K |
| RPS burst | 10K | 100K | 1.1M |
| CPU req/limit | 2 / 4 cores | 20 / 40 | 220 / 440 |
| Memory req/limit | 4 / 8 GiB | 40 / 80 | 440 / 880 |
| Cedar cache entries / pod | 100K | 1M | 11M |

10K RPS per region capacity covers the 100K/day-new-agreement workload at 10× headroom.

### 2.2 enforcement-app (separate deployment for hot-path isolation)

| Metric | Per-pod | 30 pods/region | 11 regions |
|--------|---------|----------------|------------|
| RPS sustained | 10K | 300K | 3.3M |
| p99 latency | ≤10ms | ≤10ms | ≤10ms (per-region) |
| Cache hit rate | ≥80% | ≥80% | ≥80% |
| Cedar evaluator pool | 50 threads | 1500 | 16500 |

3.3M evaluations/s aggregate. Peak demand 100K/s globally → 33× headroom.

### 2.3 revocation-app

| Metric | Per-pod | 5 pods/region | 11 regions |
|--------|---------|----------------|------------|
| Originated rev/s sustained | 1K | 5K | 55K |
| Outbox drain throughput | 5K msg/s | 25K | 275K |
| Pulsar publish p99 | ≤50ms | ≤50ms | ≤50ms |

Year-1 peak 1M rev/day = ~12 rev/s mean; 50× spike covers DSAR cascade scenarios.

### 2.4 projection-gateway-worker

| Metric | Per-pod | 20 pods/region | 11 regions |
|--------|---------|----------------|------------|
| Project events/s | 50K | 1M | 11M |
| Narrow + emit p99 | ≤100ms | ≤100ms | ≤500ms (incl Pulsar) |

100B/day = ~1.16M/s mean; peak 5M/s during business hours; 11M global capacity = 2× headroom.

### 2.5 audit-bridge-worker

| Metric | Per-pod | 10 pods/region | 11 regions |
|--------|---------|----------------|------------|
| Bilateral emits/s | 5K | 50K | 550K |
| p99 seal latency | ≤500ms | ≤500ms | ≤500ms |

50B/day = ~580K/s mean. Capacity = 550K/region × 11 = 6M/s aggregate; 10× headroom.

## 3. Storage (Postgres + Citus)

### 3.1 `consent_graph_agreements`
- Row size: ~4KB (jsonb scope + terms + sovereignty)
- 10M rows × 4KB = 40GB per region (with grantor-region sharding → ~3.6GB per region average; KR
  region likely higher due to KR-grantor concentration).
- With 16 worker nodes per region Citus cluster: ~2.5GB per worker.
- Index overhead ~2× row size → 80GB peak per region.

### 3.2 `consent_graph_revocations` + `revocation_receipts`
- 1M revocations/day × 365 = 365M rows year-1.
- 30-day hot retention → 30M rows = 30GB.
- Older rows archived to S3 (replay-able via backfill-replay runbook).

### 3.3 `consent_graph_cross_pointers`
- 50B events/day → 18T rows/year — UNSUSTAINABLE in Postgres.
- Resolution: hot table holds only 30-day rolling window; older entries derive directly from
  audit-chain query API on demand.
- 30-day hot: 1.5T rows × 200B = 300TB → still too large for Postgres.
- Final design: cross-pointers stored *only* for high-stakes event classes (grant / accept / amend /
  revoke / partner-handshake) — ~10K/s globally = 300M/30d = 60GB hot. Routine events (project-emit /
  project-read) cross-pointer'd directly in audit-chain entries themselves (no separate table).

### 3.4 `consent_graph_partner_tenants`
- Bounded by partner pairs: 10K peers × 10K partners = 100M pairs theoretical; realistically ≤1M
  pairs.
- ~2KB/row → 2GB per region.

### 3.5 `consent_graph_compiled_policies`
- 10M agreements × 5KB compiled artifact = 50GB per region.
- Used only on cold-start; not hot-path.

### 3.6 `consent_graph_dp_budget`
- One row per agreement (10M).
- ~256B/row → 2.5GB per region.

### Total Postgres footprint per region
≈ 200GB year-1; 2TB year-3.

## 4. Pulsar capacity

### 4.1 Topics
- 10M projection topics + 1 revocation topic + 1 audit-bridge topic + ~10 admin/control topics
  = ~10M topics global, ~1M per region.
- Pulsar broker topic limit: 100K stable per broker → 10 brokers per region just for topic count
  capacity.
- BookKeeper bookie disk: 100B events/day × 200B/event × 7d retention = 140TB per region in BookKeeper.

### 4.2 Throughput
- Projection emission: ~1M msg/s/region × 11 regions = 11M/s aggregate.
- Pulsar broker throughput at 2-broker quorum: ~500K msg/s/broker → 4 brokers per region.
- Total brokers per region: max(topic-capacity-driven=10, throughput-driven=4) = 10 brokers.

### 4.3 Network
- Per-event payload ~1KB; per-region egress ~1GB/s peak.
- Cross-region geo-replication only for revocation + audit-bridge topics (~10K msg/s ≈ 10MB/s) →
  negligible relative to projection topic traffic.

## 5. CPU + memory totals (year-1 peak per region)

| Service | Pods | CPU req (cores) | Memory req (GiB) |
|---------|------|------------------|------------------|
| consent-graph-app | 10 | 20 | 40 |
| enforcement-app | 30 | 60 | 120 |
| revocation-app | 5 | 10 | 20 |
| projection-gateway-worker | 20 | 40 | 80 |
| audit-bridge-worker | 10 | 20 | 40 |
| partner-directory-app | 2 | 4 | 8 |
| consent-graph-worker | 5 | 10 | 20 |
| **subtotal per region** | **82** | **164 cores** | **328 GiB** |
| **× 11 regions** | **902** | **1804 cores** | **3608 GiB** |

Plus Postgres + Pulsar substrate (multi-tenanted with other µservices, shared cost).

## 6. Cost model

See `cost-budget.md` for $$$ rollup.

## 7. Auto-scaling

- enforcement-app HPA: scale on `request_qps` ≥ 8K/pod (80% of 10K capacity).
- projection-gateway-worker HPA: scale on `pulsar_subscriber_consumer_unacked` > 1K.
- audit-bridge-worker HPA: scale on `audit_bridge_outbox_oldest_seconds` > 30s.
- revocation-app: largely static; scale on `outbox_unpublished_count_seconds_oldest` > 5s.
- consent-graph-worker (background): scale on `cross_pointer_reconciliation_pending` > 100K.

## 8. Bottleneck analysis

| Component | Likely bottleneck | Mitigation |
|-----------|-------------------|------------|
| Cedar evaluation | thread pool exhaustion | 50→100 threads/pod via `tokio::task::spawn_blocking` for compile |
| Pulsar publish | broker queue depth | priority lane on revocation; HPA on broker count |
| Postgres cross-region write | RTT × commit fsync | minimize cross-region writes; route to home region |
| Citus distribution | shard skew on hot grantor | proactive rebalance + per-grantor agreement cap (10K) |
| OpenBao key fetch | TTL miss storm at restart | warm-cache OpenBao reads at startup; 1h TTL |
| Audit-chain seal latency | substrate-shared with other µservices | HG-AUDIT capacity planning shared budget |

## 9. Year-2/3 growth

- 10× agreement growth → +200 enforcement-app pods + +20 broker per region.
- Storage growth: 200GB → 2TB Postgres → standard Citus scale (linear).
- Cost scales sub-linearly (cache hit rate stays ≥80% → marginal enforcement cost low).

## 10. Verification

- Bench `criterion` suite stamps current p99 latencies at each release.
- Capacity test in staging: 50% of year-1 projected load × 1h sustained → no SLO burn.
- Quarterly load test in production canary region (1 pod) → extrapolate.
