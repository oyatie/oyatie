# PHASE-01 — Analytics OLAP Bootstrap

**Status:** Planned (introduced 2026-05-18 by data-substrate batch)
**Owner:** council-analytics
**Authority ADRs:** ADR-0193 (canonical), ADR-0184, ADR-0195

## Phase scope

Stand up the analytics µservice's ClickHouse cluster, per-tenant bootstrap, ingest pipeline, and first three tenant-visible dashboard surfaces. This is the first phase; subsequent phases extend with additional dashboard verticals, advanced rollup primitives, and the in-house Phase-2 substrate per ADR-0193 §"In-house roadmap".

## Phase exit criteria

- ClickHouse cluster live in dev cell + KR cell (production-shape).
- Per-tenant database bootstrap controller reconciles tenant-onboarded events.
- Outbox→ClickHouse CDC pipeline running with <5s ingest lag p99.
- First three tenant-facing dashboards (workflow execution metrics, audit-log search, billing rollup) live.
- Self-SLO 4-window burn-rate alerts wired through to PagerDuty + Opsgenie.
- Cold-tier S3 disk verified at 1TB and 100TB scale.
- Backup + restore drill complete per ADR-0152 RPO/RTO canonical.

## IP sequence

15 implementation packets; each is file-level granular; each is independently shippable behind a feature flag (per ADR-0159).

| IP | Title | Owner | Depends on |
|---|---|---|---|
| IP-001 | ClickHouse cluster IaC | infra | — |
| IP-002 | Per-tenant database bootstrap | infra + tenancy | IP-001 |
| IP-003 | OLAP client kernel wiring + adapter crate scaffold | backend | — |
| IP-004 | Outbox → ClickHouse CDC ingest pipeline | backend | IP-002, IP-003 |
| IP-005 | Materialized View canon (default stream tier) | backend | IP-004 |
| IP-006 | Cold-tier S3 disk + TTL retention | infra | IP-001 |
| IP-007 | Tenant-facing dashboard API (REST + GraphQL) | backend | IP-002, IP-005 |
| IP-008 | Audit-log query API (filter + cursor pagination) | backend | IP-005 |
| IP-009 | Billing rollup pipeline | backend | IP-004, IP-005 |
| IP-010 | Cross-cell federation via Distributed engine | infra | IP-001 |
| IP-011 | Per-tenant quota enforcement | backend | IP-002 |
| IP-012 | Backup tool + restore drill | infra | IP-001 |
| IP-013 | Regulator-export evidence pack | backend | IP-008 |
| IP-014 | Self-SLO authoring + alerts | sre | IP-001 |
| IP-015 | App composition root + REST/gRPC adapters | backend | IP-003, IP-007, IP-008, IP-009 |

## Cross-cutting

- **Cedar policy.** Per ADR-0007 — every external API path is Cedar-authorized. Policy fragments at `microservices/analytics/policy/`.
- **Audit chain.** Per ADR-0003 — every external query emits an audit event.
- **OpenSLO.** Per ADR-0186 Stage 5 — SLOs authored at `microservices/analytics/slos/*.openslo.yaml`.
- **In-house roadmap.** This phase ships the ClickHouse-adapter path; the Phase-2 in-house `oya-olap-warehouse-server` per ADR-0193 §"In-house roadmap" is a separate future phase.

## Risks

1. **Per-tenant database scaling.** ClickHouse database catalog at >10K databases per cluster has historical performance issues. Mitigation: per-cell sharding — each cell hosts a subset of tenants; cross-cell federation handles global views.
2. **MV ingest lag during burst.** Mitigation: per-MV target table is `AggregatingMergeTree`; backpressure handled by Pulsar consumer offset.
3. **Cold-tier S3 read latency for older queries.** Mitigation: aggressive caching at the dashboard layer; cold-tier reads are rare by design (90-day hot window covers >95% of queries).
