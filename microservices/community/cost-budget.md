---
doc_class: CostBudget
template_id: TPL-COST-BUDGET
microservice: community
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-community + ops-finance
related_adrs: [ADR-0056, ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

# Cost Budget: community µservice

## Sizing Tiers

| Tier | Tenants | Members / tenant | Posts / day / tenant | KB articles / tenant | Hot search QPS |
|---|---|---|---|---|---|
| XS | 1 – 100 | 100 | 200 | 50 | 5 |
| S | 100 – 1 k | 500 | 1 k | 200 | 25 |
| M | 1 k – 10 k | 2 k | 5 k | 500 | 200 |
| L | 10 k – 100 k | 10 k | 20 k | 2 k | 1 500 |
| XL | 100 k+ | 50 k | 100 k | 10 k | 10 000 |

## Per-Tier Monthly Compute Budget

| Component | XS | S | M | L | XL |
|---|---|---|---|---|---|
| Postgres + Citus (workers + coordinator) | $400 | $1 200 | $4 800 | $18 000 | $72 000 |
| Elasticsearch (data + master + ingest) | $300 | $900 | $3 600 | $14 000 | $56 000 |
| Valkey (hot-feed + vote buffer) | $80 | $240 | $1 000 | $4 000 | $16 000 |
| S3 (KB attachment store, 5 TB → 500 TB) | $115 | $345 | $1 380 | $5 500 | $22 000 |
| Worker fleet (reindex / guardrails-bridge / audit-chain-seal) | $200 | $600 | $2 400 | $9 000 | $36 000 |
| REST / SDK gateways | $100 | $300 | $1 200 | $4 500 | $18 000 |
| ClamAV inline scanner | $50 | $150 | $600 | $2 400 | $9 600 |
| OpenBao read traffic | $20 | $60 | $240 | $960 | $3 840 |
| Audit-chain seal write traffic | $30 | $90 | $360 | $1 440 | $5 760 |
| Observability (per-µservice signal) | $50 | $150 | $600 | $2 400 | $9 600 |
| **Total / month** | **$1 345** | **$4 035** | **$16 180** | **$62 200** | **$248 800** |

## Per-Action Unit Costs (rough)

| Action | Compute | Notes |
|---|---|---|
| Post create | $0.000 03 | Postgres insert + search async + audit-chain seal |
| Post read (feed hit) | $0.000 002 | Valkey hot-feed hit |
| Post read (cold miss) | $0.000 02 | Postgres + warm cache fill |
| Vote cast | $0.000 008 | Valkey SET NX + async Postgres flush |
| Search query | $0.000 05 | Elasticsearch fanned across shards |
| KB article publish (5 MB attachment) | $0.005 | S3 PUT + ClamAV scan + sha256 + audit-chain seal |
| Moderation action | $0.000 04 | Postgres + audit-chain seal + audit log |

## Budget Alerts

- **80 % monthly burn** at 25th of the month → page on-call.
- **100 % monthly burn** any day → page on-call + page tenant_admin if a single tenant is dominant.
- **Anomaly detection** — sudden tenant burn > 5× baseline triggers `runbooks/spam-flood-throttle.md`.

## Cost-Saving Levers

1. **Per-tenant compaction window** — KB attachments older than 90 d move from S3 standard to S3 IA (saves ~40 % on cold storage).
2. **Valkey LFU eviction** — replace LRU with LFU; saves ~15 % Valkey memory.
3. **Elasticsearch index-lifecycle** — hot (7 d) → warm (30 d) → cold (90 d) → frozen (indefinite).
4. **Worker right-sizing** — quarterly capacity review; HPA per worker fleet.
5. **Per-tenant chargeback** — XL tenants billed at usage; XS bundled in subscription.

## Forecast (12 m horizon)

| Quarter | Expected tier mix | Forecast spend / month |
|---|---|---|
| Q3 2026 | 20 XS / 5 S / 1 M | $60 k |
| Q4 2026 | 40 XS / 15 S / 3 M / 1 L | $150 k |
| Q1 2027 | 70 XS / 30 S / 8 M / 2 L | $310 k |
| Q2 2027 | 100 XS / 50 S / 15 M / 5 L / 1 XL | $700 k |
