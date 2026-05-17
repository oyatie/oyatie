---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: messenger
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-messenger + cloud-iac + cloud-k8s
deciders: ops-sre-reliability, axis-messenger, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0126, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/messenger/policy/data-residency.md
  - microservices/messenger/capacity-model.md
  - microservices/messenger/cost-budget.md
  - microservices/messenger/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (messenger µservice)

## Purpose

Define multi-region topology for messenger across the 11 oyatie packs: pack-pinning, in-pack DR pair, cross-pack replication-forbidden policy, BCDR posture, RPO/RTO targets, failover procedures.

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
│  │ S3 attachment bucket     │◀──────────▶│ S3 replica                │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐            ┌──────────────────────────┐ │
│  │ Tantivy index (rebuilt   │            │ Tantivy warm (replays     │ │
│  │ from Postgres)           │            │ message-stream events)    │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│                                                                       │
│  Global Traffic Manager (per-pack DNS):                               │
│  - Health check on primary's gateway + Postgres write path            │
│  - On failure: DNS failover → DR pair (≤ 60s TTL)                     │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region |
|---|---|---|---|
| Postgres messages + channels + threads | Async logical replication | ≤ 5 s | intra-pack only |
| Redis presence + read-receipt | Cluster-replicated (sentinel) | ≤ 1 s | intra-pack only |
| S3 attachments | Async CRR | ≤ 5 min | intra-pack only |
| Tantivy search index | Rebuilt from event stream in DR | ≤ 30 min lag during failover | intra-pack only |
| Audit-chain seals | Cross-pack OK (no PII; just hashes) | ≤ 10 s | yes |

## RPO + RTO Targets

| Tier | RPO | RTO | Notes |
|---|---|---|---|
| Message store (professional) | ≤ 5 s | ≤ 15 min | Postgres logical-replica |
| Message store (personal-DM E2E ciphertext) | ≤ 5 s | ≤ 15 min | same |
| Attachment store | ≤ 5 min | ≤ 1 h | S3 CRR + lazy hydration |
| Presence | ≤ 30 s (data lossy; rebuildable from active connections) | ≤ 5 min | from connection re-establishment |
| Read-receipts | ≤ 5 min (best-effort) | ≤ 15 min | from Redis warm + re-emit |
| Search index | ≤ 30 min lag | ≤ 1 h rebuild | replay from message stream |
| Audit-chain seals | ≤ 10 s | ≤ 1 min | cross-pack replicable |

## DR Failover Procedure

| Step | Action | Time |
|---|---|---|
| 1 | Primary fails health-check (gateway + Postgres write) sustained ≥ 5 min | – |
| 2 | Incident Commander declares Sev-1; engages OpsLead | ≤ 5 min |
| 3 | Verify DR pair Postgres replica < 30s lag; promote replica to primary | ≤ 5 min |
| 4 | DNS TTL drains; clients reconnect to DR pair gateways | ≤ 10 min total |
| 5 | Verify Redis warm cluster ready; rebuild presence from active connections | ≤ 5 min |
| 6 | Verify S3 CRR replica reachable; lazy-hydrate cold-tier on demand | ≤ 5 min |
| 7 | Replay Tantivy index from message-stream event log (last 24h hot) | ≤ 30 min |
| 8 | Verify ACL audit-chain consistency: re-derive from authoritative replay | ≤ 30 min |
| 9 | Comms: notify tenants of degraded state during failover | continuous |
| 10 | Postmortem within 5 business days | – |

Total time budget: ≤ 35 min for message-send + WebSocket + presence to be green in DR; ≤ 1h for full feature parity.

## Single-Region Packs

For packs without DR pair (pack-kr, pack-jp, pack-sg):

- **Backup**: cross-AZ snapshots every 6h; off-region encrypted-snapshot backup every 24h to a "neighbor" pack of the same legal jurisdiction (still within-pack legally; uses a separate cloud region within the same jurisdiction's borders).
- **Recovery**: from-snapshot restore; estimated RTO 4–8h; RPO ≤ 6h.
- **Rationale**: regulatory single-region constraint; no DR pair in same jurisdiction.

## Cross-Pack Replication: Forbidden

Per `policy/data-residency.md`. No messenger data crosses pack boundaries except:

- Audit-chain seals (hash-only; no PII).
- Personal-DM E2E ciphertext on user device (not server-mediated cross-pack flow).
- Public OpenAPI / AsyncAPI schemas.

Any cross-pack-replication attempt triggers `messenger_pack_residency_violation_total` (target = 0); Sev-1 alert.

## Active-Active Within Pack

- WebSocket gateways: active-active across AZs in primary region.
- Postgres: primary + 2 read-replicas across AZs.
- Redis: 3-node cluster across AZs.
- S3: cross-AZ replication within bucket.
- Tantivy: shard-per-AZ.

## Chaos Drills

| Drill | Cadence | Owner |
|---|---|---|
| Primary Postgres failover | Quarterly | ops-sre-reliability |
| WebSocket gateway pod-eviction storm | Quarterly | axis-messenger |
| Redis cluster split-brain | Annually | ops-sre-reliability |
| Pack-wide DR failover | Annually (DR-pair packs only) | ops-sre-reliability |
| Cross-context routing chaos (synthetic violation attempt) | Quarterly | ops-security |

## References

- ADR-0117.
- Parallel ADR-0126.
- `microservices/messenger/policy/data-residency.md`.
- `microservices/messenger/capacity-model.md`.
- `microservices/messenger/cost-budget.md`.
- `microservices/observability/multi-region.md` (shape reference).
- OCI multi-region docs.
