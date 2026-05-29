---
doc_class: Multi-Region
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0248
  - ADR-0241
  - ADR-0276
companion_docs:
  - microservices/ops-dashboard-control-center/ARCHITECTURE.md
  - microservices/ops-dashboard-control-center/capacity-model.md
planned_enforcement_ref: oya-governance-microservice-doc-set
---

# Multi-Region — ops-dashboard-control-center

Hyperscaler precedent: **AWS Control Tower** multi-region admin surface; **Google Cloud Console** global-but-regionally-scoped admin access.

## §1 Cell deployment map

| Cell | Region | Tier | Sovereigns served |
|---|---|---|---|
| `cell-us-east-1-control` | us-east-1 | Tier 1 | US, CA, LATAM, AE, KSA |
| `cell-eu-west-1-control` | eu-west-1 | Tier 1 | EU, UK, CH, IL |
| `cell-ap-northeast-2-control` | ap-northeast-2 | Tier 1 | KR |
| `cell-ap-northeast-1-control` | ap-northeast-1 | Tier 1 | JP, SG, AU, IN |

Each cell is an independent Tier-1 control-plane deployment. Operators are routed to their home-region cell. Cross-region admin actions require explicit Cedar context `cross_region_justification` field.

## §2 Replication topology

Control-plane events (incident records, deployment approvals, rollback decisions) replicated via Kafka MirrorMaker 2:
- Replication lag SLO: ≤5s P99 (per manifest `slos/operator-action-audit-completeness.openslo.yaml`).
- Direction: active-active cross-region replication for read-path; single-home for mutation origin.
- Conflict resolution: HLC timestamp + origin-cell wins on concurrent mutation (rare; T3 mutations are human-paced).

## §3 Failure modes

| Failure | Impact | Handling |
|---|---|---|
| Single cell unreachable | Operators in that region cannot reach home cell | DNS failover to nearest cell; read-only mode; mutations queued in outbox |
| Cross-region replication lag > SLO | Stale posture data in remote cells | Alert on `replication_lag_seconds > 5`; UI shows staleness indicator; reads still served |
| Control-plane database primary failure | Mutations blocked in affected cell | Patroni failover ≤30s; outbox drains after promotion |
| Full region loss | All operators in region routed to DR-pair cell | DR-pair: `us-east-1` ↔ `eu-west-1`; `ap-northeast-2` ↔ `ap-northeast-1`; data-residency honoured via pack overlay |

## §4 Sovereign-cell constraints

- **EU operators** (`oya-pack-eu`): processed exclusively in `cell-eu-west-1-control`. No audit log cross-border transfer to `us-east-1`. GDPR Art. 46 SCCs required for any cross-border export.
- **KR operators** (`oya-pack-kr`): processed exclusively in `cell-ap-northeast-2-control`. K-ISMS access log stays in KR.
- **US operators with FedRAMP**: processed in `cell-us-east-1-control` US-Gov zone; no cross-border.

## §5 RTO / RPO

| Tier | RTO | RPO | Authority |
|---|---|---|---|
| Hot (mutations) | 300s | 60s | `manifest.json rpo_rto` per ADR-0180 |
| Read-path (posture views) | 60s (cached) | 30s (freshness SLO) | `slos/cluster-health-freshness.openslo.yaml` |

## §6 Backup portability

Per ADR-0276: per-region backup exported as signed JSONL zstd archive. Restoration test: quarterly drill per `runbooks/admin-action-rollback.md §recovery-drill`. See `backfill-replay.md §portability`.
