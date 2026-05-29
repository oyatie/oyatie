---
doc_class: MultiRegion
title: "Multi-region deployment plan"
microservice: developer-sdk
status: Accepted
owner_team: axis-ecosystem
date: 2026-05-18
related_adrs: [ADR-0213, ADR-0131]
doc_status: published
---

# Multi-region deployment plan


## Topology

| Region | Pack | Status | Primary substrate |
|---|---|---|---|
| us-east-1 | us, us-healthcare, us-financial | Phase-1 GA | Postgres + Valkey + OpenBao primary |
| us-gov-east-1 | us-public-sector | Phase-3 GA | GovCloud-isolated |
| eu-central-1 | eu | Phase-2 GA | EU-isolated; no cross-border |
| ap-northeast-2 | kr | Phase-2 GA | KR-isolated; no cross-border |
| ap-southeast-1 | sg, au, in | Phase-4 | Asia-Pacific |
| sa-east-1 | br | Phase-4 | South America |

## Replication

- Catalog: read-replica per region; nightly cross-region sync via SeaweedFS.
- Installations: per-region authoritative; no cross-region replication of tenant data.
- Audit chain: per-region; cross-region chain consolidation in audit-chain µservice.
- Payout: per-pack rail; no cross-pack routing.

## Failover

- Cilium L4 traffic policy routes to primary; failover to nearest replica region within 60s if primary unreachable.
- Cross-region failover requires manual approval per ADR-0202.

## Latency budgets

- Tenant in pack-kr: TTI ≤ 2s served from ap-northeast-2; ≤ 3s from failover region (us-east-1).
- Developer worldwide: dev portal TTI ≤ 1s from nearest edge.

