---
doc_class: ImplementationPlan
ip_id: IP-009
title: ClickHouse history-mirror (outbox → Kafka → ClickHouse OLAP)
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology
date: 2026-05-17
depends_on: [IP-008]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-foundry-fitness-ontology-dynamic-freshness
  - oya-foundry-fitness-shardability
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-entity-store-adapter-clickhouse/
  - microservices/ontology/src/crates/oya-ontology-query-engine-adapter-clickhouse/
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: ClickHouse history-mirror

## Intent

Author the ClickHouse history-mirror backend-qualified adapters. Outbox → Kafka → ClickHouse mirror ingester rebuilds OLAP-ready rows from canonical Postgres writes; OLAP Function reads route through ClickHouse for analytics latency. Per ADR-0105 Amendment 3 `*-adapter-<backend>` pattern.

## Scope

In-scope:
- `oya-ontology-entity-store-adapter-clickhouse`: outbox-consumer worker; writes mirror rows to ClickHouse `ReplicatedMergeTree` tables partitioned by `(tenant_id, toYYYYMM(ts))`.
- `oya-ontology-query-engine-adapter-clickhouse`: ClickHouse OLAP query implementation; row-policies enforced for per-tenant scope.
- ClickHouse schema migrations under `iac/helm/clickhouse/migrations/`.
- Per-tenant ClickHouse row policies + per-tenant `max_memory_usage` quota.
- Mirror-lag SLO ≤ 60 s p99; metric `clickhouse_mirror_lag_seconds`.

## Implementation

| Step | Action |
|---|---|
| 1 | Scaffold 2 adapter-clickhouse crates |
| 2 | Author ClickHouse schema migrations (mirror tables matching every Object Type) |
| 3 | Author outbox-consumer worker (reads Kafka `ontology.events.object-instance-mutated.v1`; writes to ClickHouse staging; promotes) |
| 4 | Author OLAP query adapter (ClickHouse client; tier-filter projection) |
| 5 | Wire per-tenant row policies via ClickHouse `CREATE ROW POLICY` |
| 6 | Author mirror-lag SLO + alert |
| 7 | Tests: outbox replay rebuilds mirror; cross-tenant query refused; freshness ≤ 60 s |

## Verification

- Mirror-lag p99 ≤ 60 s in synthetic load test.
- ClickHouse row policy refuses cross-tenant query.
- `oya gate validate ontology-dynamic-freshness --microservice ontology` — exit 0.

## References

- ADR-0105 Amendment 3 (`*-adapter-<backend>` pattern).
- Bominal ADR-0050 (outbox).
- ClickHouse — `clickhouse.com/docs/en/engines/table-engines/mergetree-family/replication`.
- ClickHouse row policies — `clickhouse.com/docs/en/sql-reference/statements/create/row-policy`.
