# Migration playbook — AWS Aurora Postgres + DynamoDB → Oyatie `cloud-data`

Audience: a data team running production OLTP on AWS Aurora Postgres + key-value workloads on DynamoDB. Goal: migrate to
`cloud-data` (CockroachDB-backed Postgres + KV) with zero data loss and continuous availability.

## Phase 0 — Inventory (Day 0…7)

### From Aurora Postgres

1. Catalogue schemas:
   ```bash
   pg_dump --schema-only -h $AURORA_ENDPOINT -U $USER -d $DB > aurora-schema.sql
   ```
2. Measure data volume + write rate:
   ```bash
   psql -h $AURORA_ENDPOINT -U $USER -d $DB \
     -c "SELECT schemaname, relname, n_live_tup, pg_size_pretty(pg_total_relation_size(relid)) FROM pg_stat_user_tables;"
   ```
3. Audit Postgres-specific features used: triggers, stored procedures, extensions, MVs.

### From DynamoDB

1. List tables + their indexes:
   ```bash
   aws dynamodb list-tables > dynamo-tables.json
   aws dynamodb describe-table --table-name <t> > "dynamo-table-$t.json"
   ```
2. Measure RCU/WCU + storage size.
3. Audit Streams + Global Tables usage.

## Phase 1 — Tenant + database provisioning (Day 7…14)

```bash
./bin/oya data tenant register --tenant oyatie.b2b.midmarket.acme-corp --tenant-class paid

./bin/oya data database create \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --database acme-prod \
  --engine cockroachdb-25 \
  --region us-east-2 \
  --replication-factor 3 \
  --pitr-window 30d
```

For DynamoDB workloads, decide between:
- **CockroachDB-as-KV** (recommended for most): same DB; use a single table with (tenant_id, pk, sk) primary key.
- **Dedicated KV engine**: provision a `valkey-8`-backed cache tier; only for cache-style data that doesn't need durability.

## Phase 2 — Schema translation (Day 14…35)

Run the Aurora schema-to-CRDB translator:
```bash
./bin/oya data migrate aurora-schema-to-crdb \
  --input aurora-schema.sql \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --output crdb-schema.sql \
  --add-tenant-id-pk-prefix true
```

The translator:
1. Adds `tenant_id` as the leading PK column on every table.
2. Rewrites Postgres extensions to the CRDB-compatible subset (refusing extensions not on the allowlist).
3. Translates Aurora-specific functions to CRDB equivalents (e.g. `nextval()` for sequences becomes `unique_rowid()`).
4. Flags un-portable features for manual review (`_translation_note`).

Apply schema:
```bash
./bin/oya data ddl apply \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --database acme-prod \
  --migration-file crdb-schema.sql
```

For DynamoDB, design a CRDB schema:
```sql
CREATE TABLE dynamodb_users (
  tenant_id TEXT NOT NULL DEFAULT current_setting('oya.tenant_id'),
  pk TEXT NOT NULL,
  sk TEXT NOT NULL,
  attributes JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, pk, sk)
);

CREATE INDEX dynamodb_users_gsi1 ON dynamodb_users (tenant_id, (attributes->>'email'));
```

## Phase 3 — Historical data backfill (Day 35…56)

### From Aurora

Use the AWS DMS-equivalent pipeline:
```bash
./bin/oya data migrate dms \
  --source-format aurora-postgres \
  --source-endpoint $AURORA_ENDPOINT \
  --target-tenant oyatie.b2b.midmarket.acme-corp \
  --target-database acme-prod \
  --tables customers,orders,products,... \
  --mode full-load-and-cdc \
  --concurrency 16
```

The pipeline:
1. `pg_dump`-style copies historical data (resumable).
2. Reads the Postgres logical replication slot for CDC ongoing.
3. Streams changes to CRDB in batches of 1000 rows.
4. Verifies row counts + checksums per table.

Expected throughput: ~25k rows/sec/worker × 16 workers. A 500 GB Aurora DB completes in ~6-12 h.

### From DynamoDB

```bash
./bin/oya data migrate dynamo \
  --source-table acme-users \
  --target-tenant oyatie.b2b.midmarket.acme-corp \
  --target-database acme-prod \
  --target-table dynamodb_users \
  --pk-mapping "pk:id;sk:type" \
  --batch-size 1000 \
  --enable-streams-cdc true
```

DynamoDB Streams ingestion picks up writes during the migration window.

## Phase 4 — Dual-write phase (Day 56…84)

For each application service, configure dual-write:
```rust
let db = DualWrite::new()
    .primary(LegacyAuroraClient::new(...))
    .secondary(oya_cloud_data_sdk::CrdbClient::connect(cfg).await?)
    .strategy(DualWriteStrategy::WriteBoth_ReadPrimary);
```

Reads continue to come from Aurora; writes go to both. The Oyatie side is shadow-only for the first 2 weeks.

Reconciliation check:
```bash
./bin/oya data migrate dual-write-divergence \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --since "24h ago" \
  --tolerance-row-count 0.001 \
  --tolerance-checksum 0
```

## Phase 5 — Cut-over (Day 84…112)

Per service, flip the SDK config:
```rust
let db = DualWrite::new()
    .primary(oya_cloud_data_sdk::CrdbClient::connect(cfg).await?)  // <- swap
    .secondary(LegacyAuroraClient::new(...))
    .strategy(DualWriteStrategy::WriteBoth_ReadPrimary);  // <- now reads from Oyatie
```

Verify reads are coming from `cloud-data`:
```bash
./bin/oya data query-log query \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --since "1h ago" \
  --group-by query_kind
```

Keep Aurora as a hot standby for 14 d.

## Phase 6 — Decommission Aurora + DynamoDB (Day 112…140)

After 30 d clean run on Oyatie:
1. Disable dual-write (read+write all on Oyatie).
2. Take a final Aurora snapshot (retained per SOX-404 7 y).
3. Delete Aurora cluster (`aws rds delete-db-cluster`).
4. Take a final DynamoDB backup.
5. Delete DynamoDB tables.

## Rollback strategy

Within Phase 4 dual-write:
- Switch SDK back to `WriteBoth_ReadPrimary` with Aurora as primary.
- Cost: rollback latency ~30 s per service deploy.

After Phase 5 cut-over:
- Switch SDK back to Aurora-primary; Aurora is still receiving writes.
- Manually reconcile any writes that only went to Oyatie.
- Plan: 4-8 h depending on data volume.

After Phase 6 decommission: Aurora snapshot is the only restore path; restore takes 2-8 h.

## What you gain

- 17-58 % TCO reduction vs Aurora + DynamoDB combo at mid-market scale.
- Bundled OLTP + OLAP + ledger + cache + graph engines.
- Per-tenant CMK envelope encryption.
- Cedar policy authority on every query.
- HLC default + TrueTime opt-in per workload.
- Multi-engine horizontal scale (CRDB beats Aurora-Global write-forwarding by 10×).
- BLAKE3 audit chain + per-tenant compliance pack overlays.

## What you give up

- DynamoDB internet-scale ceiling (we max at ~5 M QPS per cell vs DynamoDB's effectively-unlimited).
- AWS service integration depth (Lambda, Glue, EMR, etc. wire to Aurora natively).
- AWS Backup orchestration (Oyatie's backup is per-µservice).
- Public self-service signup; you need a tenant + tenant_class.
- Some Postgres extensions (no full PostGIS at paid tenant_class; pgvector requires paid tenant_class).
