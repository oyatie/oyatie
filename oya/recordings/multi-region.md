---
doc_class: MultiRegionPlan
template_id: TPL-MULTI-REGION
microservice: recordings
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-recordings
related_adrs: [ADR-0117, ADR-0135, ADR-0131, ADR-RECORDINGS-0004, ADR-RECORDINGS-0005]
doc_status: published
---

# Multi-Region Plan: recordings µservice

## Topology

| Pack | Primary OCI region | DR pair | CDN backend |
|---|---|---|---|
| pack-kr | ap-seoul-1 | none (KR-resident only) | self-host (Bunny + Fastly + nginx-vod per ADR-RECORDINGS-0004) |
| pack-eu | eu-frankfurt-1 | eu-amsterdam-1 | CloudFront (EU edge) |
| pack-us | us-ashburn-1 | us-phoenix-1 | CloudFront |
| pack-us-healthcare | us-ashburn-1 (HIPAA-eligible) | us-phoenix-1 (HIPAA-eligible) | CloudFront (HIPAA-eligible edges only) |
| pack-us-financial | us-ashburn-1 | us-phoenix-1 | CloudFront + S3 object-lock per SEC 17a-4 |
| pack-jp | ap-tokyo-1 | ap-osaka-1 | CloudFront (JP edge) |
| pack-sg | ap-singapore-1 | (none — single region) | CloudFront |
| pack-au | ap-sydney-1 | ap-melbourne-1 | CloudFront |
| pack-in | ap-hyderabad-1 | ap-mumbai-1 | CloudFront |
| pack-br | sa-saopaulo-1 | sa-vinhedo-1 | CloudFront |
| pack-ae | me-abudhabi-1 | me-dubai-1 | CloudFront |
| pack-ksa | me-jeddah-1 | me-riyadh-1 | self-host (PDPL residency strict) |

## Replication policies

- **Postgres**: logical replication primary → warm-standby within the pack
  (DR pair). RPO ≤ 1 min. Cross-pack replication forbidden.
- **S3 hot**: cross-AZ replication within the pack. RPO ≤ 5 min. Cross-pack
  forbidden.
- **S3 cold (Glacier-class)**: per ADR-RECORDINGS-0005 — same-pack only.
  Pack-us-financial: S3 object-lock (WORM) per SEC 17a-4(f).
- **Valkey**: primary-replica HA in-pack. Share-link cache + playback session
  rebuildable from Postgres on failover.
- **Meilisearch**: snapshot DR within pack (primary-only; snapshot every
  6h to S3-cold; on restore, replay transcript Workflow events to
  reconstruct).
- **Audit-chain seals**: cross-pack-portable (no PII; Merkle root + signature
  only).

## Failover Procedure

| Step | Action | RTO target |
|---|---|---|
| 1 | Health check fails on primary | ≤ 30s detection |
| 2 | Postgres logical-replication promotion of warm-standby | ≤ 5 min |
| 3 | S3 endpoint flipped to DR-pair bucket | ≤ 30s (DNS TTL + Route53 health-check) |
| 4 | CDN origin re-pointed via Lambda@Edge | ≤ 30s |
| 5 | Valkey HA promotion | ≤ 30s |
| 6 | Meilisearch restored from latest snapshot | ≤ 30 min |
| 7 | foundry-runtime (Whisper + pyannote) is regional-native (no DR replay needed) | n/a |
| 8 | Audit-chain seals re-anchored on DR | ≤ 5 min |

Total RTO: ≤ 15 min for metadata + ≤ 30 min for media + ≤ 30 min for search.

## Cross-Pack Federation

Forbidden by default. Tenant-opt-in federation (for multinational tenants
that want a unified archive across, e.g., pack-eu and pack-us) is a
future ADR; ingest contract is shaped to support it but residency rules
block it today.

## DR Test Cadence

- Quarterly DR drill per pack (game-day).
- Annual full-region failover test on a non-prod cell.
- DR drill results stored in evidence/.

## References

- ADR-0117 — residency.
- ADR-RECORDINGS-0004 — CDN strategy.
- ADR-RECORDINGS-0005 — storage tiering.
- `runbooks/playback-cdn-cache-cascade.md`.
- `policy/data-residency.md`.
