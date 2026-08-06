# Data Engineer — First Week on `cloud-data`

Audience: a data engineer / DBA with Postgres + CockroachDB + DynamoDB + MongoDB + ClickHouse experience joining the
`oya-cloud-data-*` lane. Goal: by Friday EOD you can provision a tenant logical DB, choose a consistency model, create a
geo-partitioned table, take a PITR snapshot, and walk a multi-engine workload.

## Day 1 — read before touching

- `docs/decisions/ADR-0700-ci-admission-live-apex.md` — per-cell data plane; blast radius.
- `docs/decisions/ADR-0252-hlc-default-truetime-tier.md` — HLC vs TrueTime tradeoff.
- `docs/decisions/ADR-0702-identity-authz-live-apex.md` — tenant-id PK prefix.
- ADR-0329, ADR-0330, and ADR-0331 tenant_class model — the two tenant classes and engine availability.
- CockroachDB v25 docs (multi-region patterns, geo-partitioning, follower reads).
- Spanner TrueTime paper (OSDI 2012) — at minimum understand `commit_wait`.

Clone:
```bash
./bin/oya git worktree-add --base dev --branch onboarding/$USER-data-week1 .worktrees/$USER-data-week1
cd .worktrees/$USER-data-week1
```

## Day 2 — bring up a loopback data cell

```bash
make dev-cell.up CELL=data-loopback-1 PROFILE=cloud-data-dev
make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid
```

Provision a tenant logical DB:
```bash
./bin/oya data database create \
  --tenant oyatie.b2b.smb.acme-software \
  --database acme-prod \
  --engine cockroachdb-25 \
  --region us-east-2 \
  --replication-factor 3 \
  --pitr-window 14d
```

Expected:
```
database_id      : db-2026-05-20-...
endpoint         : pg://acme-prod.cloud-data.loopback.oyatie.local:26257/acme-prod
encryption_cmk   : cmk-acme-default (from cloud-kms tenant CMK)
replication      : 3× synchronous within us-east-2
pitr_window      : 14 d
backup_schedule  : every 6 h to s3://oyatie-backup-paid-us-east-2/...
audit_chain_event: ce-2026-05-20T10:01:33Z-…
```

(via the SDK; direct psql is refused):
```bash
./bin/oya data connect \
  --tenant oyatie.b2b.smb.acme-software \
  --database acme-prod
```

You're now in a tenant-scoped session. Try a raw SELECT without tenant scope:
```sql
SELECT * FROM information_schema.tables;
```

Expected: returns only the tenant's tables (the SDK injects an implicit `WHERE tenant_id = ...` for all queries — the row-level
filter is enforced by Cedar at the proxy).

## Day 3 — schema + first writes

Author a simple schema:
```sql
CREATE TABLE customers (
  tenant_id TEXT NOT NULL DEFAULT current_setting('oya.tenant_id'),
  id        UUID NOT NULL DEFAULT gen_random_uuid(),
  email     TEXT NOT NULL,
  name      TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, id),
  UNIQUE (tenant_id, email)
);

CREATE TABLE orders (
  tenant_id   TEXT NOT NULL DEFAULT current_setting('oya.tenant_id'),
  id          UUID NOT NULL DEFAULT gen_random_uuid(),
  customer_id UUID NOT NULL,
  amount_cents INTEGER NOT NULL,
  status      TEXT NOT NULL,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, id),
  FOREIGN KEY (tenant_id, customer_id) REFERENCES customers(tenant_id, id)
);
```

Insert + select:
```sql
INSERT INTO customers (email, name) VALUES ('jane@example.com', 'Jane Doe') RETURNING id;
INSERT INTO orders (customer_id, amount_cents, status) VALUES ('<jane-id>', 9900, 'pending');

SELECT c.email, o.amount_cents FROM customers c JOIN orders o ON c.id = o.customer_id;
```

## Day 4 — consistency model + geo-partition

Switch to bounded-stale read for a dashboard query:
```sql
SET cloud_data.read_consistency = 'bounded-stale:200ms';
SELECT COUNT(*) FROM orders WHERE status = 'pending';
SET cloud_data.read_consistency = 'strong';
```

Bounded-stale reads can be served by any follower replica, including cross-region replicas — orders of magnitude cheaper than
synchronous reads for non-critical paths.

Convert the `orders` table to geo-partitioned (paid tenant_class; on paid tenant_class it's emulated as a single partition):
```sql
ALTER TABLE orders PARTITION BY LIST (tenant_id, region);
CREATE PARTITION orders_us EU('us') OF orders FOR VALUES IN (('oyatie.b2b.smb.acme-software', 'us'));
CREATE PARTITION orders_eu EU('eu') OF orders FOR VALUES IN (('oyatie.b2b.smb.acme-software', 'eu'));
```

(At paid tenant_class the partition CRDB feature is on; paid tenant_class uses constraints instead.)

## Day 5 — PITR + backup + multi-engine

Take a manual snapshot (the schedule already produces one every 6 h):
```bash
./bin/oya data snapshot create \
  --tenant oyatie.b2b.smb.acme-software \
  --database acme-prod \
  --reason "week-1 tutorial sample"
```

Restore to a fresh database at a specific point in time:
```bash
./bin/oya data pitr restore \
  --tenant oyatie.b2b.smb.acme-software \
  --source-database acme-prod \
  --target-database acme-prod-restored \
  --restore-to "2026-05-19T14:00:00Z"
```

Verify the restored DB has data only up to the restore point.

Multi-engine warm-up — attach a Valkey cache to your DB:
```bash
./bin/oya data valkey-cache enable \
  --tenant oyatie.b2b.smb.acme-software \
  --database acme-prod \
  --size 1GB \
  --eviction allkeys-lru
```

Use it via the SDK:
```rust
use oya_cloud_data_sdk::cache::ValkeyCache;
let cache = ValkeyCache::for_database("acme-prod").await?;
cache.set("session:42", b"{\"user\":\"alice\"}", Duration::from_secs(900)).await?;
let v = cache.get("session:42").await?;
```

For OLAP queries at paid tenant_class, enable a ClickHouse OLAP tier:
```bash
./bin/oya data olap-tier enable \
  --tenant oyatie.b2b.smb.acme-software \
  --database acme-prod \
  --engine clickhouse-24 \
  --replicate-tables orders,customers \
  --sync-cadence 5m
```

## What "done with week 1" means

- [ ] You can recite the two tenant classes and which engines + consistency models each unlocks.
- [ ] You created a tenant database with the canonical PK convention.
- [ ] You used at least two consistency models (`strong` + `bounded-stale`).
- [ ] You took a PITR snapshot and restored it.
- [ ] You enabled the Valkey cache + walked a use case.
- [ ] You read ADR-0248 + ADR-0252 + ADR-0244 + the Spanner TrueTime paper.

## Rookie traps

1. **Forgetting `tenant_id` in PKs.** New tables without `tenant_id` in the PK get refused at `oya data ddl apply`; the `lean-a3-tenant-trace`
   lane catches it.
2. **Using strong consistency for dashboards.** Strong consistency is expensive; use bounded-stale for read-mostly aggregations.
3. **Long-running transactions.** CRDB locks rows; transactions > 30 s in production are anti-pattern. Use saga / multi-statement
   workflow patterns.
4. **Schema migration without backup-then-test.** Every DDL change must be Sandbox-restore-tested before applying to live. Use
   `oya data ddl test --against-pitr <ts>`.
5. **Skipping replica-promotion ceremony.** Promoting a cross-region read replica to leader requires Cedar permit + reviewer-agent;
   force-promotion can split-brain.
6. **TrueTime opt-in without understanding `commit_wait`.** TrueTime adds 7-10 ms per commit; opt in only when external consistency
   is required (fin ledger, regulatory audit).
