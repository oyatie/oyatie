---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: meet
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-meet + cloud-iac + cloud-k8s
deciders: ops-sre-reliability, axis-meet, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/meet/policy/data-residency.md
  - microservices/meet/capacity-model.md
  - microservices/meet/cost-budget.md
  - microservices/meet/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (meet µservice)

## Purpose

Define multi-region topology for meet across the 11 oyatie packs: pack-pinning, in-pack DR pair, cross-pack replication-forbidden policy, BCDR posture, RPO/RTO targets, failover procedures. Special considerations for live-media plane (LiveKit SFU + coturn) and recording persistence path.

## Topology Per Pack

| Pack | Primary region | DR pair (warm-standby) | Single-region? | Activation |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | YES | YES (M02 launch) |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | DR pair | Conditional |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | DR pair | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | OCI us-phoenix-1 (HIPAA-eligible) | DR pair; isolated | Conditional (post-BAA) |
| pack-us-financial | OCI us-ashburn-1 (FedRAMP-eligible) | OCI us-phoenix-1 (FedRAMP-eligible) | DR pair; isolated | Conditional (SEC/FINRA gating) |
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
│  │ LiveKit SFU StatefulSet  │            │ SFU warm-standby          │ │
│  │ Active (handling media)  │            │ 0.5× capacity             │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐            ┌──────────────────────────┐ │
│  │ coturn TURN cluster      │            │ TURN warm-standby         │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐ logical    ┌──────────────────────────┐ │
│  │ Postgres primary + 2 RR  │◀──────────▶│ Postgres logical-replica  │ │
│  │ HA-RF=3                  │ replic     │ async; ≤ 5 s lag          │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐            ┌──────────────────────────┐ │
│  │ Redis cluster (3 nodes)  │            │ Redis warm                │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐  CRR       ┌──────────────────────────┐ │
│  │ S3 recording bucket      │◀──────────▶│ S3 replica                │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐            ┌──────────────────────────┐ │
│  │ Whisper GPU pool         │            │ Whisper warm GPU pool     │ │
│  │ (per-region; not repl)   │            │ (cold-spare; spin-up 5min)│ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐            ┌──────────────────────────┐ │
│  │ Meilisearch              │            │ Meilisearch (replays      │ │
│  │ (rebuilt from S3)        │            │ from S3 transcripts)      │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│                                                                       │
│  Global Traffic Manager (per-pack DNS):                               │
│  - Health check on primary's meet-rest + LiveKit + Postgres           │
│  - On failure: DNS failover → DR pair (≤ 60s TTL)                     │
│  - Active media sessions: NO seamless mid-call failover (clients      │
│    must rejoin; LiveKit room state lost in primary outage)            │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region |
|---|---|---|---|
| Postgres meeting + participant + recording manifest | Async logical replication | ≤ 5 s | intra-pack only |
| Redis lobby + presence + signaling session | Cluster-replicated (sentinel) | ≤ 1 s | intra-pack only |
| S3 recordings + transcripts | Async CRR | ≤ 5 min | intra-pack only |
| Meilisearch transcript index | Rebuilt from S3 transcripts in DR | ≤ 60 min lag during failover | intra-pack only |
| LiveKit room state | NOT replicated (in-memory; ephemeral) | active calls drop on failover | n/a |
| Audit-chain seals | Cross-pack OK (no PII; just hashes) | ≤ 10 s | yes |

## RPO + RTO Targets

| Tier | RPO | RTO | Notes |
|---|---|---|---|
| Meeting metadata (room, participants, recording manifest) | ≤ 5 s | ≤ 15 min | Postgres logical-replica |
| Active media sessions (LiveKit room state) | ephemeral; lost on failover | clients reconnect ≤ 30s | LiveKit room state in-memory; tradeoff for stateless SFU |
| Recording blobs | ≤ 5 min | ≤ 1 h | S3 CRR + lazy hydration |
| Live caption sessions | reconnect required after failover | ≤ 5 min | Whisper warm-spare spin-up |
| Transcript backfill | ≤ 1 h | rebuilt from S3 | per `backfill-replay.md` |
| Search index | ≤ 60 min lag | ≤ 2 h rebuild | replay from S3 transcripts |
| Audit-chain seals | ≤ 10 s | ≤ 1 min | cross-pack replicable |

## DR Failover Procedure

| Step | Action | Time |
|---|---|---|
| 1 | Primary fails health-check (meet-rest + LiveKit + Postgres write) sustained ≥ 5 min | – |
| 2 | Incident Commander declares Sev-1; engages OpsLead | ≤ 5 min |
| 3 | Verify DR pair Postgres replica < 30s lag; promote replica to primary | ≤ 5 min |
| 4 | DNS TTL drains; clients reconnect to DR pair meet-rest endpoints | ≤ 10 min total |
| 5 | Notify active-meeting hosts: "Your meeting has experienced a service disruption; please rejoin via the same link" | continuous |
| 6 | Verify Redis warm cluster ready; rebuild lobby state from active connections | ≤ 5 min |
| 7 | Verify S3 CRR replica reachable; lazy-hydrate recordings on demand | ≤ 5 min |
| 8 | Spin up DR Whisper GPU pool (cold-spare → warm in ≤ 5 min) | ≤ 5 min |
| 9 | Replay Meilisearch transcript index from S3 (last 24h hot) | ≤ 60 min |
| 10 | Verify ACL audit-chain consistency: re-derive from authoritative replay | ≤ 30 min |
| 11 | Comms: notify tenants of degraded state during failover | continuous |
| 12 | Postmortem within 5 business days | – |

Total time budget: ≤ 35 min for room-create + participant-join to be green in DR; ≤ 2h for full feature parity (transcript search rebuilt).

## Single-Region Packs

For packs without DR pair (pack-kr, pack-jp, pack-sg):

- **Backup**: cross-AZ snapshots every 6h; off-region encrypted-snapshot backup every 24h to a "neighbor" pack of the same legal jurisdiction.
- **Recovery**: from-snapshot restore; estimated RTO 4–8h; RPO ≤ 6h.
- **Rationale**: regulatory single-region constraint; no DR pair in same jurisdiction.

## Cross-Pack Replication: Forbidden

Per `policy/data-residency.md`. No meet data crosses pack boundaries except:

- Audit-chain seals (hash-only; no PII).
- Public OpenAPI / AsyncAPI schemas.
- Cross-pack meeting attendance (pack-eu user joins pack-us meeting) routes media through inter-region SFU mesh; recording stays in host-tenant pack only.

Any cross-pack-replication attempt triggers `meet_pack_residency_violation_total` (target = 0); Sev-1 alert.

## Cross-Pack Meeting Attendance (Special Case)

A pack-eu user joining a pack-us tenant's meeting routes their media via inter-region SFU mesh:

1. User authenticates against their home-pack (pack-eu).
2. Meet-rest issues LiveKit token scoped to the host-pack room (pack-us).
3. LiveKit pack-us SFU registers cross-region publisher; media flows pack-eu-edge → pack-us-SFU.
4. Recording (if enabled) lives in pack-us bucket; the pack-eu attendee's portion is recorded under host-tenant pack residency. Disclosure to pack-eu user data subjects accordingly.
5. SCC required when host-tenant is in GDPR-scope and attendee is EU data subject.

## Active-Active Within Pack

- meet-rest + meeting-instance + participant-worker: active-active across AZs in primary region.
- LiveKit SFU: active-active across AZs (room-affinity by hash).
- coturn: active-active across AZs (anycast).
- Postgres: primary + 2 read-replicas across AZs.
- Redis: 3-node cluster across AZs.
- S3: cross-AZ replication within bucket.
- Whisper GPU pool: distributed across AZs with GPU node selector.

## Chaos Drills

| Drill | Cadence | Owner |
|---|---|---|
| Primary Postgres failover | Quarterly | ops-sre-reliability |
| LiveKit SFU pod-eviction storm | Quarterly | axis-meet |
| coturn region failover | Quarterly | ops-sre-reliability |
| Whisper GPU pool exhaustion | Quarterly | axis-meet |
| Recording S3 outage during active meeting | Annually | axis-meet |
| Pack-wide DR failover | Annually (DR-pair packs only) | ops-sre-reliability |
| Cross-pack residency chaos (synthetic violation attempt) | Quarterly | ops-security |

## References

- ADR-0117.
- ADR-0135.
- `microservices/meet/policy/data-residency.md`.
- `microservices/meet/capacity-model.md`.
- `microservices/meet/cost-budget.md`.
- `microservices/messenger/multi-region.md` (shape reference).
- `microservices/observability/multi-region.md` (shape reference).
- OCI multi-region docs.
- LiveKit ops: room state in-memory tradeoff documented.
