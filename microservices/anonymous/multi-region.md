---
doc_class: MultiRegion
template_id: TPL-MULTIREGION
microservice: anonymous
status: Accepted
date: 2026-05-17
owner_team: axis-anonymous + ops-platform
related_adrs: [ADR-0117, ADR-0135, ADR-ANON-0006]
doc_status: published
---

# Multi-Region: anonymous µservice

## Topology

Per-pack regional cell deployment. The 11 packs map to discrete cells in distinct regulatory jurisdictions. Cells are independent (no cross-cell replication for personal-tier data).

| Pack | Primary region | Secondary region (DR) | Provider |
|---|---|---|---|
| pack-kr | ap-northeast-2 (Seoul) | ap-northeast-1 (Tokyo) — DR with KR PIPA Art. 28 attestation | OCI Seoul primary |
| pack-eu | eu-central-1 (Frankfurt) | eu-west-1 (Dublin) | AWS Frankfurt primary |
| pack-us | us-east-1 (N. Virginia) | us-west-2 (Oregon) | AWS / OCI dual |
| pack-us-healthcare | us-east-1 (N. Virginia) HIPAA-eligible | us-west-2 (Oregon) HIPAA-eligible | AWS HIPAA-eligible |
| pack-jp | ap-northeast-1 (Tokyo) | ap-northeast-3 (Osaka) | OCI Tokyo primary |
| pack-uk | eu-west-2 (London) | eu-west-1 (Dublin) | AWS |
| pack-au | ap-southeast-2 (Sydney) | ap-southeast-4 (Melbourne) | AWS |
| pack-sg | ap-southeast-1 (Singapore) | ap-southeast-2 (Sydney) | AWS |
| pack-in | ap-south-1 (Mumbai) | ap-south-2 (Hyderabad) | AWS |
| pack-br | sa-east-1 (São Paulo) | (single-region; LGPD data-residency strict) | AWS |
| pack-ae | me-central-1 (UAE) | me-south-1 (Bahrain) — limited cross-border per UAE PDPL | OCI UAE |
| pack-ksa | me-central-2 (KSA Riyadh) | (single-region; PDPL strict) | OCI KSA |

## Cross-pack data flow

**REFUSED FOR ANONYMOUS-TIER DATA.** Per I5 + ADR-ANON-0006, there is no federation. Posts in pack-kr never appear in pack-eu's feed render. The anonymous µservice is structurally single-pack-per-user.

Cross-pack flows that DO occur:
- Affinity-IdP linkage (a Bominal employee in EU who attests in pack-eu posts only on pack-eu).
- Audit-chain (per-pack audit-chain; legal-process disclosure within pack only).
- Observability ingest (metrics + traces emit cross-pack to observability µservice; opaque-handle labels only).

## RTO / RPO

| Failure scenario | RTO | RPO | Strategy |
|---|---|---|---|
| Single-AZ failure (within primary region) | ≤ 5 min | ≤ 0 (Postgres logical replication) | Auto-failover via Postgres patroni + Valkey Sentinel |
| Primary region failure | ≤ 30 min | ≤ 1 min | DR region promotion; pre-baked replicas in DR region; Postgres cross-AZ replication |
| Postgres data loss | ≤ 1h | ≤ 5 min | Point-in-time recovery from 30-day backups; rerun retention-policy worker if needed |
| Valkey cache loss | ≤ 1 min | n/a (cache only) | Cold rebuild from Postgres |
| Meilisearch index loss | ≤ 30 min | ≤ 5 min | Rebuild from Postgres |
| Blind-signature key compromise | n/a — Sev-1 event | n/a | Runbook `runbooks/blind-signature-key-ceremony.md` — emergency rotation + all in-flight credentials invalidated |
| Affinity-attestation key compromise | n/a — Sev-1 event | n/a | Runbook `runbooks/affinity-attestation-key-rotation.md` — emergency rotation + tenant-IdP renegotiation |
| Anonymity-leak (DB JOIN executed without legal-process Cedar) | n/a — P0 | n/a | Runbook `runbooks/anonymity-leak-incident-response.md` |

## DR test cadence

- Quarterly game-day per pack.
- Annually full primary-region-loss test in non-prod.
- Bi-annually blind-signature key ceremony rehearsal.
- Bi-annually legal-process disclosure tabletop (each pack rotates).

## Data residency invariants

- Per-pack pinning per ADR-0117 — hard ceiling.
- Pack-eu Personal-tier never crosses pack boundary — structural invariant (DB tenant-id check + Cedar policy).
- Pack-us-healthcare anonymous data is not commonly captured (anonymous tier is rarely PHI by design); if PHI is volunteered in post body, content-moderation removes per safe-harbor.
- Pack-kr 통신비밀보호법 data sealed per KR-PIPA Art. 24-2 alternative pseudonymous processing rules; cross-border refused except via court-ordered legal-process under Art. 9.

## Sub-region failover specifics

| Region pair | Cross-AZ replication mode | Lag p99 |
|---|---|---|
| ap-northeast-2 / ap-northeast-1 | Postgres logical async + Valkey cluster mode | ≤ 1 min |
| eu-central-1 / eu-west-1 | Postgres logical async | ≤ 30s |
| us-east-1 / us-west-2 | Postgres logical async | ≤ 1 min |
| Others | varies | ≤ 1 min |
