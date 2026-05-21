# IP-005 — Flag Adapter: Postgres (Patroni + Citus)

**microservice**: feature-flags
**bc**: flag
**layer**: adapter
**qualifier**: postgres
**crate**: oya-feature-flags-flag-adapter-postgres
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0244, ADR-0248, ADR-0252, ADR-0276
**companion_ips**: IP-003, IP-004

## Scope

Outbound port implementation for `FlagRepository` trait backed by Patroni + Citus. Tenant-sharded by `tenant_id`; 256 shards. Implements WAL-based replication path for `flag-state-changed` propagation (non-kill-switch path).

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `PostgresFlagRepository` | Implements `FlagRepository`; all queries carry `tenant_id` shard key |
| 2 | Schema migrations | `feature_flags` table: flag_key, tenant_id (DISTRIBUTION KEY), flag_type, variants JSONB, rollout_stage, sunset_at, pack_locked_fields JSONB[], hlc_created TEXT, hlc_updated TEXT |
| 3 | Connection pool | `deadpool-postgres`; pool size per replica = 20; TTL = 30s |
| 4 | WAL change listener | `pg_logical` replication slot `feature_flags_wal`; publishes decoded rows to local `FlagCacheInvalidationService` |
| 5 | DSAR export | `export_flags_for_tenant(tenant_id) -> FlagDefinitionExport` per ADR-0276; encrypted with DEK from OpenBao |
| 6 | Tests | Tenant isolation: cross-shard query returns empty; WAL listener fires within 200ms of write |

## Schema

```sql
CREATE TABLE feature_flags (
  flag_key        TEXT NOT NULL,
  tenant_id       TEXT NOT NULL,
  flag_type       TEXT NOT NULL CHECK (flag_type IN ('bool','string','number','json')),
  variants        JSONB NOT NULL DEFAULT '{}',
  rollout_stage   INTEGER NOT NULL DEFAULT 0,
  sunset_at       TIMESTAMPTZ,
  pack_locked_fields JSONB NOT NULL DEFAULT '[]',
  hlc_created     TEXT NOT NULL,
  hlc_updated     TEXT NOT NULL,
  PRIMARY KEY (tenant_id, flag_key)
) PARTITION BY HASH(tenant_id);
-- Citus: SELECT create_distributed_table('feature_flags', 'tenant_id', shard_count => 256);
```

## Definition of Done

- Migration runs idempotently on fresh + existing schema
- Connection pool recycles on 30s idle
- WAL listener test: write → cache invalidation event ≤200ms
- DSAR export test: output matches `FlagDefinitionExport` schema from openapi-v1.yaml
