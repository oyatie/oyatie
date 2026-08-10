---
doc_class: CapacityModel
microservice: cloud-secrets
status: Accepted
date: 2026-05-17
owner_team: axis-cloud-secrets + ops-sre
related_adrs: [ADR-0117, ADR-0131]
related_artifacts:
  - secrets/cost-budget.md
  - secrets/failure-modes.md
review_cadence: quarterly + on every pack activation
doc_status: published
---

# Capacity Model: cloud-secrets µservice

## Purpose

Predict cloud-secrets demand from consumer µservices' workload curves; size OpenBao + Postgres + HSM accordingly; identify scale-out triggers.

## Demand Drivers

| Driver | Unit | Source |
|---|---|---|
| Resolve qps per consumer | requests/sec | each consumer µservice's request rate × secrets-per-request count |
| Cache-hit ratio | % | SDK in-process LRU; varies with secret-diversity per consumer |
| Rotation events/day per tenant | events/day | per rotation policy cadence (KEK 1/yr, signing keys 4/yr per key, API keys 12/yr per key) |
| HSM signing ops per tenant | ops/day | rotation × cascade fanout + ad-hoc signing |
| Audit events emitted | events/sec | resolve count + lifecycle events |
| Active tenants per pack | tenants | onboarding curve |
| Active secrets per tenant | secrets | per-tenant inventory |

## Workload Model

### Per-consumer-µservice resolve qps

| Consumer µservice class | Resolves per request | Steady-state qps per replica | Notes |
|---|---|---|---|
| API-tier µservices (ontology, workflow-engine) | 2-5 per request | 200-1000 qps/replica | secrets per call: DB conn, OAuth, signing |
| Worker-tier µservices (background jobs) | 1-2 per job | 10-50 qps/replica | low per-replica rate; many replicas |
| Mesh / proxy µservices (cell, gateway) | 0 (use mTLS not secret-pull) | n/a | offloaded to cert-manager |
| Foundry µservices (agent execution) | 1-5 per agent step | 50-200 qps/replica | provider-credentials |
| Cloud-iac / cloud-k8s | 1-3 per reconcile | 5-20 qps/replica | cluster ops secrets |

### Aggregate per-pack resolve qps (pack-kr launch scenario)

- 10 µservices × 5 replicas each × 300 qps avg = 15,000 qps in-cluster
- Cache hit ratio 95% (steady state) → OpenBao backend qps = 750 qps
- Capacity headroom: OpenBao 5-node Raft cluster baseline 5,000 read qps + 500 write qps → at 750 qps backend, **6.7× headroom**.

### Aggregate per-pack rotation events/day (pack-kr launch)

- 1000 tenants × 50 secrets/tenant × 0.02 rotations/secret/day (12 per year amortised) = 1000 rotations/day
- Spread over 24h with jitter: ~12 rotations/min peak
- HSM signing ops per rotation: 2-4 (rewrap + new wrap + audit-sign)
- HSM ops/day: ~3,000/day → 35 ops/min sustained → well below 1000/s partition capacity.

## Capacity Envelope (per pack, per cluster)

| Dimension | Baseline | Headroom | Max sustained | Scale-out trigger |
|---|---|---|---|---|
| OpenBao reads (backend qps after cache) | 1,000 qps | 5× | 5,000 qps/cluster | sustained > 3,500 qps for 10 min OR p99 > 25 ms |
| OpenBao writes (rotate + revoke) | 100 qps | 5× | 500 qps/cluster | sustained > 350 qps for 10 min OR Raft leader CPU > 70% |
| Postgres TPS | 500 TPS | 4× | 2,000 TPS | Postgres CPU > 70% sustained 1h |
| HSM signing ops | 100 ops/s | 10× | 1,000 ops/s/partition | partition queue depth > 200 ms |
| Audit emission throughput | 2,000 events/s | 50× | 100,000 events/s | bridge backlog > 60s |
| Per-tenant namespaces | 100/pack baseline | 100× | 10,000/pack | namespace-controller queue lag > 10 min |
| Active secrets total | 100,000/pack | 10× | 1,000,000/pack | OpenBao KV blob storage trend |
| In-flight rotations | 10 concurrent | 10× | 100 concurrent | rotation-scheduler queue depth |

## Growth Curve Projection (12-month, pack-kr)

| Month | Active tenants | Active secrets | Resolve qps (cache-hit-adjusted) | OpenBao backend qps | Cluster sizing |
|---|---|---|---|---|---|
| M01 launch (2026-Q2) | 5 | 5,000 | 100 | 5 | 5-node baseline |
| M03 | 50 | 50,000 | 1,000 | 50 | 5-node baseline |
| M06 | 200 | 200,000 | 5,000 | 250 | 5-node baseline |
| M09 | 500 | 500,000 | 12,000 | 600 | 5-node baseline (headroom 8×) |
| M12 | 1,000 | 1,000,000 | 25,000 | 1,250 | 5-node baseline (headroom 4×); plan scale-out at M14 |

## Scale-Out Decision Tree

```text
Is sustained OpenBao backend qps > 70% of cluster capacity for 10 min?
├── No → no action
└── Yes
    ├── Is read qps > write qps?
    │   └── Add read-only Raft follower replicas (+2 nodes)
    └── Is write qps the limit?
        └── Architectural: shard via per-tenant namespace partitioning
            OR add second cluster per pack (mirror via DR pair)
```

## Per-Tenant Capacity Limits (multi-tenant policy)

OpenBao Sentinel + quota policies bound per-tenant capacity:

| Limit | Tier `sandbox` | Tier `trial` | Tier `production-small` | Tier `production-medium` | Tier `production-large` | Tier `production-regulated` |
|---|---|---|---|---|---|---|
| Max secrets per tenant | 100 | 1,000 | 10,000 | 100,000 | 1,000,000 | 10,000,000 |
| Resolve qps per tenant | 10 | 100 | 1,000 | 10,000 | 100,000 | 1,000,000 |
| Rotations/day per tenant | 5 | 50 | 500 | 5,000 | 50,000 | 500,000 |
| encryption-key BYOK upload events/month (ADR-0251 §D-10) | 0 | 0 | 5 | 20 | 100 | 1,000 |
| HSM partition | shared (sandbox HSM) | shared | shared | shared | shared | dedicated |

Tenants approaching limits receive quota-warning events 7d ahead; exceeding triggers throttling + sales-engineering ticket for tier upgrade.

## Self-Observability SLO Targets (resolver hot path)

| SLI | Target | Window | Burn-rate alarm |
|---|---|---|---|
| availability (success rate) | ≥ 99.99 % | 30d rolling | 14.4× over 1h |
| latency p99 (cache-hit) | ≤ 10 ms | 30d rolling | 7.2× over 6h |
| latency p99 (cache-miss) | ≤ 25 ms | 30d rolling | 7.2× over 6h |
| audit emission completeness | ≥ 99.95 % | 30d rolling | 14.4× over 1h |
| rotation SLA conformance | ≥ 99.5 % | 30d rolling | per scheduler |

## References

- `secrets/cost-budget.md`
- `secrets/failure-modes.md`
- `secrets/PRD.md`
- `secrets/observability/slos/cloud-secrets/*.openslo.yaml` (authored under observability gate)
- OpenBao performance benchmarks (informing OpenBao expectations)
- OCI Cloud-HSM published throughput
