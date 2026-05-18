---
doc_class: MultiRegionPlan
template_id: TPL-MULTI-REGION
microservice: drive
status: Accepted
date: 2026-05-17
owner_team: axis-drive + ops-sre-reliability
related_adrs: [ADR-0117, ADR-0135, ADR-0130, ADR-0131, ADR-0140, ADR-DRIVE-0001, ADR-DRIVE-0006]
doc_status: published
---

# Multi-Region Plan — drive µservice

## Purpose

Define data-residency posture, cross-region replication topology, failover semantics, and pack-pinning enforcement for the drive µservice.

## Residency posture

Tenant bytes pinned to the tenant's pack region per ADR-0117 + ADR-0140. Cross-region replication forbidden by default. Cross-region replication enabled only with tenant-executed SCCs + tenant-DPA per Arts. 44–46 / KR PIPA Art. 17.

| Pack | Primary region | Secondary region (DR; same pack) | Cross-region eligible |
|---|---|---|---|
| pack-kr | ap-seoul-1 | ap-seoul-2 | NO (within KR only) |
| pack-eu | eu-frankfurt-1 | eu-amsterdam-1 | NO (within EU only) |
| pack-us | us-east-1 | us-west-2 | NO (within US only) |
| pack-us-healthcare | us-east-1-hipaa | us-west-2-hipaa | NO (HIPAA region only) |
| pack-jp | ap-tokyo-1 | ap-osaka-1 | NO |
| pack-sg | ap-singapore-1 | ap-jakarta-1 | NO (with PDPA assessment) |
| pack-au | ap-sydney-1 | ap-melbourne-1 | NO |
| pack-in | ap-mumbai-1 | ap-hyderabad-1 | NO (DPDPA whitelist-based) |
| pack-br | sa-east-1 | sa-saopaulo-2 | NO (ANPD-approved only) |
| pack-ae | me-dubai-1 | me-abudhabi-1 | NO |
| pack-ksa | me-riyadh-1 | me-jeddah-1 | NO (SDAIA-approved only) |

## Replication topology

### Within-pack replication

| Component | Strategy | RPO | RTO |
|---|---|---|---|
| Object store (Garage / MinIO / SeaweedFS) | replication-factor 3 within-pack; sync across primary + secondary region | ≤ 60s | ≤ 15min |
| Postgres metadata | logical replication primary→secondary; sync replicas in primary region | ≤ 30s | ≤ 5min |
| Redis (upload session + sync cache) | not replicated cross-region; reconstructable on failover | tolerated loss; sessions re-issued | n/a |
| Meilisearch (full-text index) | sync across primary + secondary region | ≤ 60s | ≤ 30min |
| Audit-chain seal records | replicated via audit-chain µservice (out-of-scope here) | per audit-chain | per audit-chain |

### Cross-pack replication (off by default)

Only enabled when tenant executes SCCs + DPA explicitly. Backup-restore use-case only; not live failover. Cross-pack replication uses an explicit `cross-pack-replication-grant` Cedar policy; refuses without it.

## Failover

| Trigger | Action | Owner |
|---|---|---|
| Primary region object-store cell loss | auto-rebuild on neighbour cells; runbook `object-storage-degraded.md` | ops-sre-reliability |
| Primary region Postgres failure | promote secondary region replica; DNS swing; runbook `postgres-failover.md` (in `cloud-iac/`) | ops-sre-reliability |
| Primary region full-region outage | manual failover to secondary region within pack; DNS swing; runbook `region-failover.md` | ops-sre-reliability + axis-drive |
| Pack-level outage (multi-region within pack) | declare disaster; engage backup-restore per `backfill-replay.md`; tenant comms | council-architecture + ops-sre-reliability |

## DR drills

| Drill | Cadence | Owner |
|---|---|---|
| Single-cell object-store loss | quarterly | ops-sre-reliability |
| Single-region failure → secondary swing | semi-annually | ops-sre-reliability |
| Full pack-level failure → backup-restore | annually | council-architecture |
| Cross-pack tenant-specific restore (SCC-gated) | annually | council-privacy + axis-drive |

## Pack-pinning enforcement

Pack-pinning is enforced at three layers:
1. **Ingress layer**: per-pack DNS / per-pack edge proxy refuses requests carrying a different pack claim.
2. **Cedar policy layer**: `policy/data-residency.md` refuses cross-pack reads/writes by default; SCC-gated permit clause.
3. **LEAN check**: `oya-check-pack-pinning` refuses build if any drive crate hard-codes a cross-pack route.

## References

- ADR-0117 — Cloud-native infrastructure / data residency.
- ADR-0140 — Cedar policy enforcement.
- ADR-DRIVE-0001 — Object-storage substrate selection (Garage replication topology).
- ADR-DRIVE-0006 — Immutability + WORM policy (replication respects WORM semantics).
- `microservices/drive/policy/data-residency.md`.
- `microservices/drive/runbooks/object-storage-degraded.md`.
- Garage replication docs; MinIO replication docs; SeaweedFS replication docs.
