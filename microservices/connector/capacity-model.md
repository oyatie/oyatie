---
microservice: connector
doc_class: CapacityModel
date: 2026-05-20
owner_team: axis-integration + ops-sre-reliability
status: Accepted
related_adrs: [ADR-0145, ADR-0248, ADR-0253]
doc_status: published
---

# Capacity Model — connector (Integration Substrate)

## Workload assumptions

- 1,000 tenants at GA; 50,000 at 24mo.
- Median tenant: 10 active wirings × 100 actions/day = 1k actions/tenant-day.
- P95 tenant: 100 wirings × 1k actions/day = 100k actions/tenant-day.
- Webhook traffic: 100M receives/day platform-wide at GA (Zapier baseline: ~3B/day; we target 3% of that within 24mo).
- OAuth grants in-flight: 100k concurrent peak.

## Derivations

### Per-worker capacity (connector-adapter-worker)

- Per-action overhead budget: 5ms p50, 20ms p99 (excludes vendor RTT).
- Single-worker capacity (Little's Law): N = λ × W → 1 worker @ p99=20ms with W=20ms can handle λ = 1/0.020 = 50 RPS sustained, ≈ 4.3M actions/day.
- Target: 100k actions/sec platform-wide → 100,000/50 = **2,000 workers** at peak.

### Webhook receiver capacity

- Receive overhead p99 = 100ms (HMAC verify + enqueue).
- Single edge instance: 1/0.100 = 10 RPS sustained per fingerprint. With 100 concurrent connections per instance: 1,000 RPS/instance.
- Target: 100M receives/day = 1,157 RPS average; peak 10× = 11,570 RPS → **12 instances** + headroom = 24 instances at peak.

### OAuth grant storage (Postgres)

- 100k concurrent grants × 1KB/grant = 100MB working-set in PG.
- Lifetime grants (5yr retention for audit): 100k × 5yr × refresh rate = ~5M rows; with 1KB each = 5GB. Trivial.

### Valkey (rate-limit + idempotency)

- Token-bucket per (tenant, connector, action) = 50k × 500 × 50 = 1.25B keys. Per-key overhead 100B → 125GB. Sharded across 16 Valkey instances per pack.
- Idempotency-key dedup: 100M/day × 24h TTL = 100M keys/pack. Per-key 200B → 20GB/pack. Manageable.

### Shuffle-sharding (ADR-0248)

- M=64 Valkey shards per pack; per-tenant pin to N=4 shards.
- P(two-tenants-share-all-N-shards) = C(M-N,N)/C(M,N) = C(60,4)/C(64,4) = 487,635 / 762,376 ≈ 0.64 (this is wrong; correct formula:)
- P(noisy-neighbor cascade to ≥X% of tenants) using shuffle-sharding formula per Hyperscaler Architecture Invariants doc: with N=4, M=64, the blast radius of a single bad tenant is ≤4/64 = 6.25% of shards. Combined with per-tenant token-bucket isolation, effective blast radius ≤0.1% of tenants share enough shards to be impacted.

## Bottleneck identification

| Bottleneck | When | Mitigation |
|---|---|---|
| Adapter-worker pool exhaustion | Peak burst > 2x average | HPA on `oya_connector_action_queue_depth`; min=200, max=4000 workers per pack |
| Valkey token-bucket hot key | Single tenant exceeds quota | Per-tenant 429 with `Retry-After`; circuit-break vendor on sustained 429s |
| Vendor API rate-limit (e.g., Salesforce daily limit) | Heavy tenant hits Salesforce's 100k/day | Per-tenant + per-vendor quota gate via Cedar; surface to tenant dashboard |
| HMAC verification CPU | 100M webhooks/day × constant-time SHA256 | Constant-time implementation in Rust; AES-NI / SHA-NI accelerated; ~50µs per verify; trivially headroom |
| OpenBao TTL renewal storms | 60s access token TTL × 100k concurrent | Sidecar batches renewals; token cache with lazy refresh; budget: 1.6k OpenBao reads/sec/pack |

## Tail-latency mitigation

Per Tail at Scale (Dean & Barroso 2013):
- Hedged requests for read-only catalog queries: duplicate after p95 budget (~150ms); first response wins.
- Circuit-breakers per ADR-0145 §invariant-1: open after 3 consecutive 5xxs; half-open after 30s.
- Outlier detection in Envoy: consecutive_5xx=3, base_ejection_time=30s.

## Cold-start budget

- New tenant first connector wiring: <60s end-to-end (OAuth dance is the long pole, ~20s).
- New connector adapter (loaded on first use): <5s for adapter binary fetch from marketplace registry + Kata sandbox spin-up.

## Multi-region awareness

Per ADR-0248 cellular architecture:
- Tier-0 edge cells: webhook-receiver + oauth-broker (geo-routed via DNS GSLB).
- Tier-1 cells: connector-adapter-worker (per-region per-pack).
- Tier-2 cells: dlq-replay-worker (regional).
- Tier-3 data cells: not used (connect needs internet egress).

Failover RTO ≤15min; RPO ≤60s (one DLQ flush cycle).

## References

- ADR-0145 inter-microservice communication reform §invariant-1
- ADR-0248 cellular architecture (shuffle-sharding formulas)
- `docs/standards/hyperscaler-architecture-invariants.md`
