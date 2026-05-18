---
doc_class: CapacityModel
template_id: TPL-CAPACITY-MODEL
microservice: identity
status: Accepted
date: 2026-05-18
owner_team: axis-identity + ops-sre-reliability
---

# Capacity Model — identity µservice

## Workload classes

| Class | Operations | p99 latency target | Saturation signal |
|---|---|---|---|
| Hot-path verify | OIDC token verification (JWKS-cache-hit) | ≤2ms | in-process cache miss rate |
| Hot-path issue | OIDC token issuance | ≤80ms | Postgres write IOPS |
| WebAuthn auth | passkey assertion finish | ≤100ms | Zitadel CPU |
| WebAuthn register | passkey registration finish | ≤250ms | Zitadel CPU + Postgres write |
| SCIM mutate | POST/PATCH/DELETE | ≤500ms | Postgres write IOPS |
| SCIM list (paginated) | GET Users/Groups | ≤300ms | Postgres read replica CPU |
| Step-up grant | ACR upgrade flow (UX-bound) | ≤8s | end-user time |

## Per-pack workload assumptions

### Year-1 (100K users globally, 9K-10K per pack average)

| Metric | Steady-state | Burst peak |
|---|---|---|
| Token verifications/sec (read-mostly) | 1,000 rps per pack | 5,000 rps (login surge) |
| Token issuances/sec | 50 rps per pack | 500 rps |
| WebAuthn authentications/sec | 30 rps per pack | 300 rps |
| WebAuthn registrations/sec | 5 rps per pack | 50 rps (mass onboarding) |
| SCIM POSTs/sec | 2 rps per pack | 200 rps (bulk import) |
| SCIM PATCHes/sec | 5 rps per pack | 100 rps |
| Step-up grants/sec | 1 rps per pack | 30 rps |
| Audit events/sec | 100 eps per pack | 1,000 eps |

### Year-3 (1M users globally, 100K per pack average)

10× of year-1; same per-op latency targets must hold.

### Year-5 (10M users globally, 1M per pack average)

100× of year-1; some workloads horizontal-scale linearly, others (Postgres write IOPS) need re-architecture.

## Scale validation status (HONEST DISCLOSURE)

**PROJECTION vs VERIFIED**:

| Claim | Status | Validation IP |
|---|---|---|
| 1,000 token-verifications/sec/pack year-1 | PROJECTION | IP-016 load test required before pack-eu bellwether |
| 30 WebAuthn auth/sec/pack year-1 | PROJECTION | IP-016 |
| 100 SCIM POST/sec burst | PROJECTION | IP-016 |
| 10K tenants/pack year-3 | PROJECTION — Zitadel public references (Scaleway, Doctolib) do not publish at-scale tenant counts at this depth | IP-016 must validate up to 5K tenants/pack BEFORE this number is promoted from PROJECTION to TARGET. If Zitadel fails the test, ADR-0187 Phase-2 trigger criterion is met. |
| 1M users/pack year-3 | PROJECTION — Zitadel public docs claim 1M+ users per Instance; not independently verified for our workload mix | IP-016 |
| 10M users/pack year-5 | PROJECTION — at this scale Phase-2 (`oya-identity-server`) is likely the only path | IP-016 + ADR-0187 Phase-2 trigger |

**Mandatory validation gate**: pack-eu bellwether promotion is BLOCKED until IP-016 (load-test validation) reports green at YEAR-1 numbers and signals red/amber for year-3 numbers (which then drives the Phase-2 timeline decision).

## Hyperscaler 4-INV overlay (NOW, not later)

Per ADR-0186 + hyperscaler-architecture-invariants.json, identity µservice declares the four invariants in `manifest.json#hyperscaler_inv_coverage` and wires them at the kernel boundary from inception:

| Invariant | Implementation |
|---|---|
| **Circuit-breaker** | Envoy circuit-breaker on Zitadel upstream (max_connections=200; max_pending_requests=100; max_requests=400; max_retries=3); per-tenant budget split via Envoy's outlier_detection. Fail-fast on Zitadel half-open. |
| **Shuffle-sharding** | Per-tenant Postgres connection-pool partition (4 shards per pack at year-1; 16 at year-3; 64 at year-5). A noisy-neighbour tenant exhausts ≤1 shard, not the whole pool. Implemented via pgcat shuffle-sharding policy. |
| **Four-golden-signals** | rate=`oya_identity_oidc_issue_total`; errors=`oya_identity_oidc_issue_total{error=~".+"}`; latency=`oya_identity_oidc_issue_duration_seconds`; saturation=`oya_identity_zitadel_postgres_pool_utilization`. Dashboard `dashboards/identity-overview.json` panel-1 visualises all four. |
| **SLO-error-budget** | 9 SLOs declared in `manifest.json#slos`; each has a 30d rolling budget; 4-window multi-burn alert per ADR-0139. Alert routes to `axis-identity` OnCall when 2h-1d, 1h-6h, 6h-3d, or 3d-30d budget burns exceed thresholds. |

## Scaling primitives

### Zitadel pods

- Stateless beyond the Postgres event-store; horizontal-scale via HPA.
- Target: 70% CPU utilisation; 80% memory; scale-out cooldown 60s, scale-in cooldown 5min.
- Per-replica throughput envelope: 500 token-verifications/sec, 25 token-issuances/sec, 15 WebAuthn auths/sec.

| Year | Replicas / pack | Total replicas (11 packs) |
|---|---|---|
| 1 | 3 (HA floor) | 33 |
| 3 | 8 | 88 |
| 5 | 20 | 220 |

### Postgres event-store

- pgcat connection pool per ADR-0179.
- Year-1: db.r6g.xlarge (4 vCPU + 32GB) + 1 read replica.
- Year-3: db.r6g.4xlarge (16 vCPU + 128GB) + 3 read replicas + Patroni HA.
- Year-5: partition users by tenant_id shard; per-shard cluster.

### JWKS endpoint

- Read-only; serves cached document. p99 ≤8ms requires no DB hit on hot path.
- 24h JWKS-cache TTL on consumers → expected origin load: ~1 req/sec per consumer pod × N pods.
- Year-5 saturation point: estimated 50K rps; Envoy edge cache absorbs 99% reaching origin at 500 rps — well within capacity.

### Audit emitter

- Async to `audit-chain` µservice.
- Buffer + DLQ; emitter throughput: 5,000 eps per pod.
- Year-5: 1 pod sufficient per pack assuming `audit-chain` keeps up.

## Quotas + rate limits

| Surface | Per-tenant default | Per-IP default | Burst |
|---|---|---|---|
| `/oauth/v2/token` | 100 rps | 10 rps | 5× for 10s |
| `/oauth/v2/userinfo` | 200 rps | 20 rps | 5× for 10s |
| `/oauth/v2/keys` (JWKS) | unlimited (cached 24h) | 1000 rps | unlimited |
| `/webauthn/*` | 50 rps | 5 rps | 3× for 10s |
| `/scim/v2/{tenant}/Users` POST | 100 rps | n/a | 10× for 1min (bulk import burst) |
| `/scim/v2/{tenant}/Users` GET (list) | 50 rps | n/a | 3× for 10s |
| `/scim/v2/{tenant}/Users/{id}` PATCH | 200 rps | n/a | 5× for 10s |

Tenants needing higher quotas request via enterprise-tier procurement.

## Failure-mode capacity

| Scenario | Degraded throughput |
|---|---|
| 1 of 3 AZs down | 67% of normal (still > 100% of steady-state at year-1) |
| Postgres primary failover (≤30s blip) | 0% during failover; spikes after as buffered ops drain |
| FIDO-MDS3 fetch failed (>48h stale) | unchanged (uses local cache); regulated packs may refuse novel AAGUIDs |
| Audit-chain µservice down | emitter buffers up to 1M events per pod before backpressuring (≤30 min budget) |

## Scale-out triggers

| Signal | Action |
|---|---|
| Zitadel CPU > 70% sustained 5min | scale-out replicas +25% |
| Zitadel memory > 80% | scale-out replicas +25% |
| Postgres connection-pool saturation > 80% | scale read-replicas; alert FE for query-pattern review |
| JWKS endpoint p99 > 10ms | investigate (likely Envoy cache miss; not DB) |
| WebAuthn p99 > 200ms | investigate (CBOR parse cost; webauthn-rs upgrade or pod scale) |
| SCIM endpoint p99 > 800ms | investigate (Postgres query plan; index audit) |

## Hardware envelope (per pack year-5 projection)

| Component | Sizing |
|---|---|
| Zitadel pods | 20 × (4 vCPU + 8GB) |
| Postgres primary | 64 vCPU + 512GB + 8TB NVMe |
| Postgres replicas | 3 × 32 vCPU + 256GB |
| pgcat pools | 3 × (2 vCPU + 4GB) |
| Audit emitter | 1 × (1 vCPU + 1GB) |
| AAGUID refresh worker | 1 × (0.25 vCPU + 256MB) |
| HRIS poller | per tenant: 0.5 vCPU + 512MB; sized per active HRIS integrations |

## Year-5 saturation re-architecture

If year-5 actuals exceed projections, options in priority order:
1. Tenant-shard Postgres (per-tenant or per-tenant-cluster cluster).
2. Move JWKS to CDN-pinned static asset (24h TTL).
3. Phase 2 (in-house `oya-identity-server`) — already triggered by ≥10K tenants or active-active requirement (per ADR-0187 §In-house roadmap).
4. Per-region Zitadel cluster split within a pack (3 AZs become 9 AZs with intra-pack sharding).
