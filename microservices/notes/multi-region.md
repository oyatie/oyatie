---
doc_class: MultiRegionPlan
title: notes µservice — Multi-Region Plan
microservice: notes
status: Accepted
date: 2026-05-17
owner_team: axis-notes + ops-sre-reliability + council-privacy
related_artifacts:
  - microservices/notes/policy/data-residency.md
  - microservices/notes/policy/dual-context-isolation.md
doc_status: published
---

# Multi-Region Plan — notes µservice

## Pack Topology

Per ADR-0117 + parallel ADR-0135:

| Pack | Primary region | DR-pair | Activation status |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | (none; warm-standby AZs only) | YES (M02 launch) |
| pack-eu | OCI eu-frankfurt-1 | eu-amsterdam-1 (warm-standby) | conditional |
| pack-us | OCI us-ashburn-1 | us-phoenix-1 (warm-standby) | conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | none | conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | (none) | conditional |
| pack-sg | OCI ap-singapore-1 | (none) | conditional |
| pack-au | OCI ap-sydney-1 | ap-melbourne-1 | conditional |
| pack-in | OCI ap-hyderabad-1 | ap-mumbai-1 | conditional |
| pack-br | OCI sa-saopaulo-1 | sa-vinhedo-1 | conditional |
| pack-ae | OCI me-abudhabi-1 | me-dubai-1 | conditional |
| pack-ksa | OCI me-jeddah-1 | me-riyadh-1 | conditional |

## Cross-Pack Posture

**Cross-pack replication: forbidden by default.** This is the same posture as messenger and stronger than docs because note content (especially Personal-tier) is often more sensitive than the user's documents.

## Within-Pack Replication

| Store | Strategy |
|---|---|
| Postgres | logical replication primary → warm-standby; sync replication for AUDIT-class rows |
| Redis | primary-replica HA within AZ pair |
| Meilisearch | per-tenant index; per-AZ replicas; rebuild from Postgres source-of-truth on restart |
| S3 (Personal-tier ciphertext blobs) | cross-AZ replication within-region only |
| Loro CRDT op-log | broker active-active per-AZ; op-log Postgres-backed |

## Failure Domains

| Domain | Blast radius | RTO | RPO |
|---|---|---|---|
| Pod | single-BC degraded | < 1 min | 0 |
| AZ | per-AZ pods + storage | < 5 min | 0 (sync) for AUDIT; ≤ 30s for body |
| Region | primary region | 15 min (failover to DR-pair where pack has one) | ≤ 5 min |
| Pack | single pack unavailable | NOT served by other packs (forbidden) | n/a |
| Loro broker | collab degraded | < 2 min | 0 (clients hold ops) |

## Failover Procedure

### Within-pack AZ failover

1. Detect AZ outage via OCI health-check.
2. Postgres warm-standby promoted to primary (5 min RTO).
3. Redis replica promoted.
4. K8s pods re-scheduled to surviving AZ within-pack.
5. Meilisearch index re-built from Postgres replay if needed (5-10 min).
6. Status page update.

### Cross-region failover (only for packs with DR-pair)

1. Primary region down for > 15 min → executive decision to fail over.
2. DR-pair already warm-standby; flip DNS routing.
3. RTO 15 min; RPO ≤ 5 min.
4. Personal-tier ciphertext blobs already cross-AZ-replicated; safe.
5. Post-failover audit-chain reconciliation if any seal in flight.

## Pack Activation Procedure

When a new pack is activated:

1. Cloud-K8s provisions cluster + DB + Redis + Meilisearch + S3 buckets in target region.
2. OpenBao mount + KMS keys per pack.
3. `iac/kustomize/overlays/pack-<id>/` patch applied.
4. Pre-launch CI gates:
   - `oya gate validate per-microservice-layout --microservice notes` exit 0
   - `oya gate validate dual-context-isolation --microservice notes` exit 0
   - `oya gate validate e2e-ai-refusal --microservice notes` exit 0
   - `oya gate validate notes-pack-residency` exit 0
5. Pack-specific gates (e.g., pack-us-healthcare requires BAA).
6. Smoke + load test (10 % of XL-max in target region).
7. Release-pointer flip.

## Personal-Tier Multi-Region Privacy

A user with Personal-tier residency assigned to pack-kr who travels to pack-us:
- User's notes remain in pack-kr Postgres + S3 (Personal residency follows user, not session).
- User's client decrypts Personal-tier ciphertext after fetching from pack-kr.
- No cross-pack data leakage; the user's device, not oyatie, traverses the network.

If user relocates and updates personal-residency to pack-us:
- Per-user migration job (background) replicates ciphertext from pack-kr to pack-us; reads from new pack going forward.
- Migration is per-user-consent (user initiates).
- Migration job emits audit-chain trail.

## Loro Collab Cross-Region

Loro collab sessions are **within-pack only**. If two users in different packs want to collaborate on a note:
- The note must reside in one pack (Professional tier).
- The other user accesses via the note's pack endpoint (cross-pack read with same Cedar scope; tenant must have cross-pack access enabled per legal-transfer mechanism).
- Op-log lives in note's pack.

## Monitoring

| Metric | Source |
|---|---|
| `oya_notes_pack_residency_violation_total` | per-tenant; alarms at > 0 |
| `oya_notes_cross_region_replication_lag_seconds` | per-pack; alarms at > 300 for AUDIT, > 600 for body |
| `oya_notes_az_failover_duration_seconds` | per-failover-event |

## References

- ADR-0117 (data residency packs).
- ADR-0064 (canonical base + localization packs).
- `policy/data-residency.md`.
- `capacity-model.md`.
