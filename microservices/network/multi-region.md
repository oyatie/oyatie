---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: network
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-network + cloud-iac + cloud-k8s
deciders: ops-sre-reliability, axis-network, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0135, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/network/policy/data-residency.md
  - microservices/network/capacity-model.md
  - microservices/network/cost-budget.md
  - microservices/network/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (network µservice)

## Purpose

Define multi-region topology for `network` across the 11 oyatie packs: pack-pinning, in-pack DR pair, cross-pack replication-forbidden policy, BCDR posture, RPO/RTO targets, failover procedures. `network` is Professional-tier only and does NOT federate in P01; residency follows the tenant (not the user). Cross-µservice bridges (messenger / mail / calendar / ATS) are pack-aligned — the bridge is in-pack only.

## Topology Per Pack

| Pack | Primary region | DR pair (warm-standby) | Single-region? | Activation |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | YES | YES (M02 launch) |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | DR pair | Conditional |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | DR pair | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | OCI us-phoenix-1 (HIPAA-eligible) | DR pair; isolated | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | — | YES | Conditional |
| pack-sg | OCI ap-singapore-1 | — | YES | Conditional |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | DR pair | Conditional |
| pack-in | OCI ap-hyderabad-1 | OCI ap-mumbai-1 | DR pair | Conditional |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | DR pair | Conditional |
| pack-ae | OCI me-abudhabi-1 | OCI me-dubai-1 | DR pair | Conditional |
| pack-ksa | OCI me-jeddah-1 | OCI me-riyadh-1 | DR pair | Conditional |

## In-Pack DR-Pair Architecture

```text
┌─ Pack <X> ────────────────────────────────────────────────────────────┐
│                                                                       │
│  Primary region                          DR-pair region               │
│  ┌──────────────────────────┐            ┌──────────────────────────┐ │
│  │ WebSocket gateway pool   │            │ Gateway warm-standby      │ │
│  │ Active (handling traffic)│            │ 0.5× capacity             │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐ logical    ┌──────────────────────────┐ │
│  │ Postgres primary + 2 RR  │◀──────────▶│ Postgres logical-replica  │ │
│  │ HA-RF=3                  │ replic     │ async; ≤ 5 s lag          │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐            ┌──────────────────────────┐ │
│  │ Redis cluster (3 shards) │            │ Redis cluster warm        │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐  CRR       ┌──────────────────────────┐ │
│  │ S3 media + doc bucket    │◀──────────▶│ S3 replica                │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐            ┌──────────────────────────┐ │
│  │ Meilisearch primary      │            │ Meilisearch warm (replays │ │
│  │ (multi-index; rebuilt    │            │ from Postgres event log)  │ │
│  │ from canonical PG)       │            │                           │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│                                                                       │
│  Cross-µservice bridges (in-pack only):                               │
│  - messenger-bridge (InMail) — talks to messenger µservice in same    │
│    pack; never crosses pack lines.                                    │
│  - calendar-bridge (events) — same pack.                              │
│  - mail-bridge (page newsletter) — same pack.                         │
│  - ATS-bridge (jobs-handoff) — Tier-G ATS µservice in same pack.      │
│                                                                       │
│  Global Traffic Manager (per-pack DNS):                               │
│  - Health check on primary's gateway + Postgres write path            │
│  - On failure: DNS failover → DR pair (≤ 60s TTL)                     │
│                                                                       │
│  Federation: NONE in P01. network does NOT federate.                  │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region |
|---|---|---|---|
| Postgres profiles + posts + connections + endorsements + jobs + pages + groups + events | Async logical replication | ≤ 5 s | intra-pack only |
| Redis feed-cache + reactions + trending + notifications + InMail queue | Cluster-replicated (sentinel) | ≤ 1 s | intra-pack only |
| S3 media + documents | Async CRR | ≤ 5 min | intra-pack only |
| Meilisearch search indexes | Rebuilt from canonical PG + event stream in DR | ≤ 30 min lag during failover | intra-pack only |
| Audit-chain seals | Cross-pack OK (no PII; just hashes) | ≤ 10 s | yes |
| Endorsement-chain Ed25519 keystore | KMS-replicated intra-pack only | ≤ 1 s | intra-pack only |

## RPO + RTO Targets

| Tier | RPO | RTO | Notes |
|---|---|---|---|
| Professional-profile store | ≤ 5 s | ≤ 15 min | Postgres logical-replica |
| Connection-graph adjacency | ≤ 5 s | ≤ 15 min | Postgres logical-replica + audit-chain replay verifier |
| Post + comment store | ≤ 5 s | ≤ 15 min | same |
| Endorsement-chain | ≤ 5 s | ≤ 15 min | Ed25519 keystore replicates first; replay verifies integrity |
| Jobs-handoff event ledger | ≤ 5 s | ≤ 5 min | contract-versioned event log; ATS bridge re-emits if needed |
| InMail-bridge queue | ≤ 30 s (best-effort) | ≤ 15 min | messenger µservice has its own DR; bridge is stateless |
| Media + document store | ≤ 5 min | ≤ 1 h | S3 CRR + lazy hydration |
| Feed cache | ≤ 30 s (data lossy; rebuildable from posts) | ≤ 5 min | from Redis warm + fanout replay |
| Reactions | ≤ 5 min (best-effort) | ≤ 15 min | from Redis warm + re-emit; Postgres flush |
| Search indexes (multi-index) | ≤ 30 min lag | ≤ 1 h rebuild | replay from canonical PG event stream |
| Trending topics | ≤ 5 min lag | ≤ 5 min rebuild | recompute from windowed hashtag events |
| Notifications | best-effort | ≤ 30 min | re-emit from event log |
| Audit-chain seals | ≤ 10 s | ≤ 1 min | cross-pack replicable (hash-only) |

## DR Failover Procedure

| Step | Action | Time |
|---|---|---|
| 1 | Primary fails health-check (gateway + Postgres write) sustained ≥ 5 min | – |
| 2 | Incident Commander declares Sev-1; engages OpsLead | ≤ 5 min |
| 3 | Verify DR pair Postgres replica < 30s lag; promote replica to primary | ≤ 5 min |
| 4 | DNS TTL drains; clients reconnect to DR pair gateways | ≤ 10 min total |
| 5 | Verify Redis warm cluster ready; rebuild feed cache from latest posts | ≤ 10 min |
| 6 | Verify S3 CRR replica reachable; lazy-hydrate cold-tier on demand | ≤ 5 min |
| 7 | Replay Meilisearch indexes from canonical PG event log (last 24h hot per index: people, content, skills, jobs, companies, events) | ≤ 30 min |
| 8 | Verify connection-graph + endorsement-chain consistency: re-derive from audit-chain authoritative replay | ≤ 30 min |
| 9 | Re-establish cross-µservice bridges (messenger / calendar / mail / ATS) — same-pack only; bridge worker re-registers | ≤ 5 min |
| 10 | Comms: notify tenants of degraded state during failover | continuous |
| 11 | Postmortem within 5 business days | – |

Total time budget: ≤ 35 min for profile-render + feed-render + connection-action + post-create to be green in DR; ≤ 1h for full feature parity (including search + trending + endorsement-chain integrity verify + InMail-bridge resumed).

## Single-Region Packs

For packs without DR pair (pack-kr, pack-jp, pack-sg):

- **Backup**: cross-AZ snapshots every 6h; off-region encrypted-snapshot backup every 24h to a "neighbor" pack of the same legal jurisdiction (still within-pack legally; uses a separate cloud region within the same jurisdiction's borders).
- **Recovery**: from-snapshot restore; estimated RTO 4–8h; RPO ≤ 6h.
- **Rationale**: regulatory single-region constraint; no DR pair in same jurisdiction.

## Cross-Pack Replication: Forbidden

Per `policy/data-residency.md`. No `network` data crosses pack boundaries except:

- Audit-chain seals (hash-only; no PII).
- Public OpenAPI / AsyncAPI schemas.
- Endorsement-chain root hashes for cross-pack replay verification (hash-only; never the underlying endorsement content).

`network` does NOT federate in P01 (no ActivityPub, no AT Protocol). If federation is added in a successor-IP ADR-NET, opt-in tenant-only + Professional-tier + SCC required.

Any cross-pack-replication attempt outside the explicit exceptions triggers `network_pack_residency_violation_total` (target = 0); Sev-1 alert.

## Active-Active Within Pack

- WebSocket gateways: active-active across AZs in primary region.
- Postgres: primary + 2 read-replicas across AZs.
- Redis: 3-node cluster across AZs.
- S3: cross-AZ replication within bucket.
- Meilisearch: shard-per-AZ; per-index sharding (people, content, skills, jobs, companies, events).

## Cross-µservice Bridge Failure Modes

- **messenger-bridge degraded**: InMail send queues to local Redis Streams; per-tenant rate limit unchanged; bridge retries with exponential backoff; surfaces alert when queue depth > 100k.
- **calendar-bridge degraded**: event RSVP and iCal emission queues; downstream calendar µservice replay on recovery.
- **mail-bridge degraded**: page newsletter sends queue; replay on recovery.
- **ATS-bridge degraded**: jobs-handoff events queue; ATS µservice (Tier G) replay on recovery; contract-versioned event log ensures idempotent re-delivery.

All cross-µservice bridges are stateless on the `network` side; failures are visible via `network_bridge_<x>_queue_depth` metrics.

## Chaos Drills

| Drill | Cadence | Owner |
|---|---|---|
| Primary Postgres failover | Quarterly | ops-sre-reliability |
| WebSocket gateway pod-eviction storm | Quarterly | axis-network |
| Redis cluster split-brain | Annually | ops-sre-reliability |
| Pack-wide DR failover | Annually (DR-pair packs only) | ops-sre-reliability |
| Connection-graph corruption + audit-replay rebuild | Quarterly | axis-network + ops-security |
| Endorsement-chain integrity verification drill | Quarterly | axis-network + axis-audit-chain |
| Feed cache rebuild after corruption | Quarterly | axis-network |
| Recommender-classifier rollback (EU AI Act high-risk) | Quarterly | axis-network + axis-foundry-runtime |
| messenger-bridge degraded drill | Quarterly | axis-network + axis-messenger |
| ATS-bridge degraded drill | Annually | axis-network + axis-ats |
| Cross-context routing chaos (synthetic Personal→Professional attempt) | Quarterly | ops-security |

## References

- ADR-0117.
- Parallel ADR-0135.
- `microservices/network/policy/data-residency.md`.
- `microservices/network/capacity-model.md`.
- `microservices/network/cost-budget.md`.
- `microservices/observability/multi-region.md` (shape reference).
- `microservices/social/multi-region.md` (sibling reference).
- OCI multi-region docs.
