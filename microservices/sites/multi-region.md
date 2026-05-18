---
doc_class: MultiRegion
template_id: TPL-MULTI-REGION
microservice: sites
status: Accepted
date: 2026-05-17
owner_team: axis-sites + ops-sre-reliability
related_adrs: [ADR-0117, ADR-0131, ADR-0133, ADR-SITES-0003, ADR-SITES-0004]
doc_status: published
---

# Multi-Region Architecture — sites µservice

## Purpose

Define how sites deploys + replicates + fails-over across regions
while honouring per-pack data residency (ADR-0117) and CDN delivery
constraints (ADR-SITES-0003).

## Regional posture

| Pack | Primary region | Failover region (within-pack only) | CDN edges |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | OCI ap-chuncheon-1 (when GA) | KR edges only |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | EU edges only |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | US edges |
| pack-us-healthcare | OCI us-ashburn-1 (BAA-eligible) | OCI us-phoenix-1 (BAA-eligible) | US edges + DR confined |
| pack-jp | OCI ap-tokyo-1 | OCI ap-osaka-1 | JP edges |
| pack-sg | OCI ap-singapore-1 | OCI ap-singapore-1 (additional AD) | SG edges |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | AU edges |
| pack-in | OCI ap-mumbai-1 | OCI ap-hyderabad-1 | IN edges |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | BR edges |
| pack-ae | OCI me-jeddah-1 / me-dubai-1 | within ME | ME edges + Hijri overlay |
| pack-ksa | OCI me-jeddah-1 | within ME | ME edges + Hijri overlay |

## Cell topology

Per cell: Postgres primary + 2 replicas (Patroni-managed); Valkey
cluster (3 shards); Meilisearch (3 instances); S3-compatible storage;
Loro CRDT relay (3 pods); per-BC rest/worker pods.

## Replication

| Layer | Strategy | RPO | RTO |
|---|---|---|---|
| Postgres | streaming + 1 sync replica + N async | ≤ 60s | ≤ 15min (Patroni failover) |
| Valkey | cluster mode replication (per-shard primary + 1 replica) | ≤ 1s | ≤ 60s |
| Meilisearch | cross-instance replication (factor 2) | ≤ 5s | ≤ 5min (reindex from Postgres if needed) |
| S3 published artifacts | within-pack cross-AD; cross-region only within pack | ≤ 60s | ≤ 5min |
| Loro CRDT log | persisted per-tenant in S3 + Postgres journal; relay reconstructs on failover | ≤ 60s | ≤ 5min |
| Audit-chain seals | inherited from audit-chain µservice multi-region posture | per audit-chain | per audit-chain |

## Cross-region constraints

- **Cross-pack replication FORBIDDEN by default.** Per
  `policy/data-residency.md` Invariant DR-02; SCC-gated when activated.
- **CDN edge selection bound to pack.** EU-resident tenants' published
  pages are served only from EU edges; geo-fenced.
- **ACME automation per-pack.** Each pack runs its own ACME client
  account pool per ADR-SITES-0004; no cross-pack cert shareability.
- **Loro CRDT relays per-pack.** Cross-pack co-editing sessions are
  refused at session-token validation.

## Failover

### Page-render path

1. CDN edge serves cache (≥ 24h survival via `stale-while-revalidate`).
2. Origin failover: cdn-delivery-rest in failover region; Postgres
   replica promoted by Patroni; S3 reads from cross-AD replica.
3. Editor write path resumes when failover region is ready (≤ 15min).

### Editor write path

1. REST routed to failover region's rest pods.
2. Postgres writes accepted on promoted primary.
3. Loro CRDT relays reattach via session-token re-validation.
4. Audit-chain emission resumes via failover region's audit-chain
   producer.

### ACME renewal path

1. domain-binding-worker in failover region picks up cert-renewal
   queue.
2. ACME challenges (DNS-01) issued from failover region's IP pool;
   DNS provider's TXT record control unchanged.
3. Issued certs propagated to cdn-delivery for load.

## Cross-cell publish

- Tenant in pack-A can publish a public-facing site whose CDN is
  global-anycast — but the origin pages live only in pack-A.
- CDN edges fetch origin from pack-A regardless of edge location;
  privacy still preserved (no published-content residency issue
  because publication is the tenant's own act).
- Editor authoring is confined to pack-A.

## DR drill

| Drill | Cadence | Owner |
|---|---|---|
| Postgres primary failover (Patroni) | quarterly | ops-sre-reliability |
| Valkey primary failover | quarterly | ops-sre-reliability |
| Meilisearch instance loss + reindex | quarterly | axis-sites |
| Full pack-region loss + failover region promotion | annually | ops-sre-reliability |
| CDN edge poisoning + signed-purge cascade | quarterly | axis-sites |
| ACME provider outage drill | annually | ops-security |
| Audit-chain seal continuity post-failover | quarterly | ops-security |

## References

- ADR-0117: cloud-native infrastructure (data residency).
- ADR-0131: per-microservice layout.
- ADR-0133: industry-best-practice.
- ADR-SITES-0003: CDN substrate.
- ADR-SITES-0004: ACME + custom domain.
- `policy/data-residency.md`, `capacity-model.md`,
  `incident-response.md`, `failure-modes.md`.
- AWS Well-Architected Reliability Pillar.
- Google SRE Workbook ch. 9 (incident management).
- Patroni HA documentation.
