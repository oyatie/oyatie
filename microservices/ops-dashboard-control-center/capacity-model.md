---
doc_class: Capacity-Model
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0248
  - ADR-0180
companion_docs:
  - microservices/ops-dashboard-control-center/ARCHITECTURE.md
  - microservices/ops-dashboard-control-center/multi-region.md
planned_enforcement_ref: oya-governance-microservice-doc-suite
---

# Capacity Model — ops-dashboard-control-center

Hyperscaler precedent: **AWS internal console** capacity model (IAM evaluations at 100k req/s per region); **Stripe Dashboard** (500 simultaneous employees; P99 <200ms reads).

## §1 Load assumptions

| Dimension | Current | 10× | 100× |
|---|---|---|---|
| Peak simultaneous operators | 500 | 5,000 | 50,000 |
| Avg requests per operator per second | 2 | 2 | 2 |
| Peak sustained req/s | 1,000 | 10,000 | 100,000 |
| Burst req/s (10s window) | 5,000 | 50,000 | 500,000 |
| T3 mutations per hour | 200 | 2,000 | 20,000 |
| Cedar evaluations per request | 1.2 (avg; some requests evaluate 2 fragments) | 1.2 | 1.2 |

## §2 Little's Law derivation

**Read path:**  
L (concurrency) = λ (arrival rate) × W (service time)  
λ = 1,000 req/s; W = 50ms (P50 read latency including Cedar eval 3ms + DB read 30ms + network 17ms)  
L = 1,000 × 0.050 = **50 concurrent in-flight requests**

At 10×: L = 10,000 × 0.050 = 500 concurrent. Pod count at 50 concurrent/pod = **10 pods** at current; **100 pods** at 10×.

**Mutation path:**  
λ = 200/3600 ≈ 0.056 T3 mutations/s; W = 2,000ms (step-up auth wait excluded; DB write + outbox + Cedar = 500ms; audit seal = 200ms; notify = 300ms; total 1,000ms P99)  
L = 0.056 × 1.0 = **0.056 concurrent mutations** — well within single-pod capacity.

## §3 Database sizing

| Resource | Current | 10× | Notes |
|---|---|---|---|
| Postgres connections (app) | 200 (PgBouncer max) | 2,000 | PgBouncer pool per pod; 5 shards |
| Audit log rows/day | 86,400 (1/s average) | 864,000 | 7yr retention = 31B rows; ClickHouse cold tier |
| Audit log storage (7yr) | ~50 GB compressed | ~500 GB | zstd compression; ClickHouse columnar |
| Ontology projection rows | 100k (incidents + approvals) | 1M | idempotent-rewrite; bounded by mutation rate |

## §4 Cedar evaluation budget

Cedar evaluation P99 target ≤5ms per request. At 1,000 req/s × 1.2 evaluations = 1,200 evals/s. Library-first evaluation (in-process); no network overhead. Single evaluation CPU time: ~0.5ms. Per-pod Cedar eval budget: 1,200 evals/s × 0.5ms = 600ms CPU/s per pod. With 4 vCPU per pod: **15% CPU for Cedar**. At 10×: 150% → scale to 10 pods (15% per pod again).

## §5 Horizontal scale-out path

HPA trigger: `oya_ops_control_center_command_queue_depth ≥ 100`. Scale-out step: +2 pods per 60s until depth < 50. Max pods: 200 (100× load). Stateless REST pods — no session affinity required (sessions in OpenBao + Valkey).

## §6 Bottleneck analysis

At 100×:
1. **PgBouncer connections**: mitigated by adding PgBouncer instances (Kubernetes HPA on connection saturation).
2. **Cedar library CPU**: mitigated by pod scale-out (Cedar eval is CPU-bound, not I/O-bound).
3. **Audit log write throughput**: mitigated by Kafka outbox (write-ahead) + async ClickHouse ingest.
4. **Step-up auth latency**: NOT a bottleneck (human-paced; ≤200 T3 mutations/hr at 100×).

System goes red when: `oya_ops_control_center_command_queue_depth > 500` sustained 5min (SLO burn triggers alert; PagerDuty page).

## §7 Cost model

| Component | Monthly cost (current) | At 10× |
|---|---|---|
| Compute (10 pods × 4 vCPU × $0.05/vCPU-hr) | $1,440 | $14,400 |
| Postgres (RDS r6g.2xlarge × 3 replicas) | $2,160 | $8,640 (r6g.8xlarge) |
| ClickHouse (audit cold tier) | $200 | $2,000 |
| OpenBao (managed) | $100 | $500 |
| **Total** | **~$3,900/mo** | **~$25,540/mo** |

Full cost breakdown in `cost-budget.md`.
