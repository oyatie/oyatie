# Tutorial — Multi-engine workload, PITR restore, opt-in to TrueTime for a ledger workload

Goal: provision OLTP + OLAP + ledger engines, perform a write/read cycle on each, take a PITR snapshot + restore, opt-in to
TrueTime for a financial ledger workload, observe `commit_wait` latency impact. Loopback `cloud-data` cell.

Pre-reqs:
- Loopback data cell: `make dev-cell.up CELL=data-loopback-1 PROFILE=cloud-data-dev`
- Tenant: `make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid`

## Step 1 — provision a multi-engine database

```bash
./bin/oya data database create \
  --tenant oyatie.b2b.smb.acme-software \
  --database acme-prod \
  --engine cockroachdb-25 \
  --region us-east-2 \
  --replication-factor 3 \
  --pitr-window 30d

./bin/oya data olap-tier enable \
  --tenant oyatie.b2b.smb.acme-software \
  --database acme-prod \
  --engine clickhouse-24 \
  --replicate-tables orders,customers,events \
  --sync-cadence 1m

./bin/oya data ledger-engine enable \
  --tenant oyatie.b2b.smb.acme-software \
  --ledger-name acme-payments \
  --engine tigerbeetle-0.16 \
  --account-batches-per-sec 10000 \
  --commit-mode deterministic-ordered
```

## Step 2 — schema + data on OLTP

Connect:
```bash
./bin/oya data connect --tenant oyatie.b2b.smb.acme-software --database acme-prod
```

```sql
CREATE TABLE customers (
  tenant_id TEXT NOT NULL DEFAULT current_setting('oya.tenant_id'),
  id        UUID NOT NULL DEFAULT gen_random_uuid(),
  email     TEXT NOT NULL,
  name      TEXT NOT NULL,
  region    TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, id),
  UNIQUE (tenant_id, email)
);

CREATE TABLE orders (
  tenant_id    TEXT NOT NULL DEFAULT current_setting('oya.tenant_id'),
  id           UUID NOT NULL DEFAULT gen_random_uuid(),
  customer_id  UUID NOT NULL,
  amount_cents BIGINT NOT NULL,
  currency     TEXT NOT NULL,
  status       TEXT NOT NULL,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, id)
);

INSERT INTO customers (email, name, region) VALUES
  ('jane@example.com', 'Jane Doe', 'us'),
  ('hans@example.de', 'Hans Müller', 'eu'),
  ('soo@example.kr', 'Park Soo', 'kr');

INSERT INTO orders (customer_id, amount_cents, currency, status)
SELECT id, (random() * 100000)::int + 1000, 'USD', 'completed' FROM customers;
```

Strong-consistency point read:
```sql
SET cloud_data.read_consistency = 'strong';
SELECT * FROM orders WHERE customer_id = '<jane-id>';
```

Latency: ≤ 6 ms p95.

Bounded-stale aggregate read:
```sql
SET cloud_data.read_consistency = 'bounded-stale:200ms';
SELECT customer_id, COUNT(*), SUM(amount_cents)
  FROM orders
  GROUP BY customer_id;
```

Latency: ≤ 1.5 ms p95 (served from a local follower replica).

## Step 3 — analytics on OLAP

After CDC sync (~1 min), query the ClickHouse OLAP tier:
```bash
./bin/oya data sql \
  --tenant oyatie.b2b.smb.acme-software \
  --database acme-prod \
  --engine clickhouse \
  --query "SELECT region, count() AS orders_count, sum(amount_cents)/100.0 AS gmv_usd FROM orders JOIN customers ON orders.customer_id = customers.id GROUP BY region ORDER BY gmv_usd DESC"
```

Expected (truncated):
```
region   orders_count  gmv_usd
us       1             456.27
kr       1             278.92
eu       1             138.45
```

## Step 4 — ledger via TigerBeetle

The ledger engine is for double-entry accounting; we don't author SQL but use the SDK directly. Sample interaction via CLI:
```bash
./bin/oya data ledger account create \
  --tenant oyatie.b2b.smb.acme-software \
  --ledger acme-payments \
  --account-id 1 --code 100 --debits-must-not-exceed-credits

./bin/oya data ledger account create \
  --tenant oyatie.b2b.smb.acme-software \
  --ledger acme-payments \
  --account-id 2 --code 100 --debits-must-not-exceed-credits

./bin/oya data ledger transfer create \
  --tenant oyatie.b2b.smb.acme-software \
  --ledger acme-payments \
  --transfer-id 100 --debit-account 1 --credit-account 2 --amount 50000 --code 200
```

Expected:
```
transfer_id      : 100
state            : posted
deterministic_order_index: 7421
commit_latency_ms: 0.42
```

## Step 5 — PITR snapshot + restore

```bash
SNAP_TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
./bin/oya data snapshot create \
  --tenant oyatie.b2b.smb.acme-software \
  --database acme-prod \
  --reason "tutorial step 5 marker"
```

Make a destructive change (we'll restore it):
```sql
DELETE FROM orders WHERE status = 'completed';
SELECT COUNT(*) FROM orders;  -- 0
```

Restore to the snapshot:
```bash
./bin/oya data pitr restore \
  --tenant oyatie.b2b.smb.acme-software \
  --source-database acme-prod \
  --target-database acme-prod-restored \
  --restore-to "$SNAP_TS"
```

Verify the restored DB has the orders:
```bash
./bin/oya data sql \
  --tenant oyatie.b2b.smb.acme-software \
  --database acme-prod-restored \
  --query "SELECT COUNT(*) FROM orders"
```

Expected: `3`.

## Step 6 — opt in to TrueTime for the ledger workload

By default, the ledger uses HLC for ordering. For a regulated workload requiring external consistency:
```bash
./bin/oya data ledger truetime enable \
  --tenant oyatie.b2b.smb.acme-software \
  --ledger acme-payments \
  --requires-cedar-permit cloud_data::Action::EnableTrueTime
```

The Cedar permit must already exist for the tenant; on the loopback profile this is auto-granted.

Compare commit latency before/after:
```bash
./bin/oya data ledger benchmark commit-latency \
  --tenant oyatie.b2b.smb.acme-software \
  --ledger acme-payments \
  --transfers 1000 \
  --report-percentiles 50,95,99
```

Expected:
```
HLC mode:
  p50:  0.42 ms   p95: 0.92 ms   p99: 1.84 ms
TrueTime mode:
  p50:  7.62 ms   p95: 8.94 ms   p99: 11.42 ms
```

The TrueTime overhead is the `commit_wait` — the engine waits for the TrueTime uncertainty bound (~7 ms) to elapse before
acknowledging the commit. This guarantees external consistency at the cost of latency.

## Step 7 — verify on audit chain

```bash
./bin/oya audit-chain query \
  --tenant oyatie.b2b.smb.acme-software \
  --kind 'cloud_data.*' \
  --since "2h ago"
```

You should see at minimum:
- `cloud_data.database.created`
- `cloud_data.olap_tier.enabled`
- `cloud_data.ledger_engine.enabled`
- `cloud_data.snapshot.taken`
- `cloud_data.pitr.restored`
- `cloud_data.truetime.enabled`

## What you just demonstrated

- Multi-engine workload: OLTP (CRDB) + OLAP (ClickHouse via 1-min CDC) + ledger (TigerBeetle deterministic ordering).
- Multiple consistency models on the same OLTP DB.
- PITR snapshot + restore with 30 d retention window.
- TrueTime opt-in with measurable `commit_wait` overhead — ADR-0252 in practice.
- BLAKE3 audit-chain anchoring across every action.
