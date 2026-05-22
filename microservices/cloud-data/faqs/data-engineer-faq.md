# `cloud-data` µservice — Data Engineer FAQ

20 real questions raised against `cloud-data` (the µservice that owns Oyatie's data substrate).

---

**Q1. Does `cloud-data` replace AWS RDS / DynamoDB / Aurora / GCP Spanner / Cloud SQL / Firestore / Azure SQL / Cosmos?**

For Oyatie-tenant workloads — yes. The underlying engines (CockroachDB for SQL+KV+document, ClickHouse for OLAP, TigerBeetle for
ledger, Neo4j for graph) cover the use-case spectrum. For tenants requiring strict-serializable cross-region writes, we offer
YugabyteDB Enterprise + TrueTime at paid tenant_class. Tenants never directly call AWS/GCP/Azure DB APIs — they use the `cloud-data` SDK.

---

**Q2. Why CockroachDB as the primary engine?**

Three reasons:
1. **Postgres-wire compat** — Postgres ecosystem (psycopg, sqlx, GORM, Hibernate) just works.
2. **Multi-region semantics** — geo-partitioned tables + follower reads + RAFT-multi-region give us Spanner-class semantics
   without TrueTime hardware.
3. **Tenant isolation** — per-tenant virtual clusters (since CRDB v24) give us logical isolation without operational overhead.

Alternative we considered: Spanner itself (requires GCP-only deployment, doesn't fit on-prem paid tenant_class); YugabyteDB (we use it
for paid tenant_class strict-serializable; CRDB covers the broader band).

---

**Q3. What's the difference between HLC and TrueTime?**

- **HLC** (Hybrid Logical Clock): combines wall-clock time + a logical counter. Causal-consistent + close-to-real-time. Used at
  demo_trial/paid tenant_class by default. Tolerates clock skew of ~500 ms gracefully.
- **TrueTime**: uses GPS + atomic clocks to bound clock uncertainty to ≤ 7 ms. Spanner uses it to enforce external consistency
  via `commit_wait` (delay commits until uncertainty elapses). Used at paid tenant_class (opt-in) and paid tenant_class (mandatory).

Per ADR-0252: HLC default; TrueTime opt-in for fin-grade and regulatory workloads.

---

**Q4. What consistency models are supported?**

5 levels, set via `SET cloud_data.read_consistency = '<level>'`:
- **strong**: linearizable read; uses the leader replica. Default. ≤ 6 ms p95.
- **bounded-stale:<duration>**: reads from any replica with bounded staleness (50 ms - 60 s). ≤ 1.5 ms p95.
- **read-as-of(hlc:<ts>)**: snapshot read at a specific HLC. Useful for replay / debugging.
- **read-as-of(truetime:<ts>)**: snapshot read with TrueTime guarantee (paid tenant_class).
- **eventual**: read from any replica with no staleness bound. Use for analytics / dashboards. ≤ 0.5 ms p95.

---

**Q5. How is per-tenant encryption handled?**

paid tenant_class — every row is envelope-encrypted with a DEK derived from a tenant CMK in `cloud-kms`. The encryption happens at the
storage layer (CockroachDB EE encryption-at-rest with per-tenant key). Cryptoshredding the CMK (via `cloud-kms`) renders the
tenant's data mathematically unrecoverable — the GDPR Art. 17 / CCPA right-to-delete primitive.

---

**Q6. What's the schema of a typical tenant table?**

```sql
CREATE TABLE <table> (
  tenant_id  TEXT NOT NULL DEFAULT current_setting('oya.tenant_id'),
  id         UUID NOT NULL DEFAULT gen_random_uuid(),
  ...columns...,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, id),
  ...
);
```

`tenant_id` first is mandatory — it ensures the PK locality keeps rows co-located per tenant + ensures cross-tenant queries
require explicit Cedar permits.

---

**Q7. How is connection pooling done?**

Each tenant has a dedicated pgbouncer pool (paid tenant_class) or shares a pool (demo_trial tenant_class). Connections are TLS-only with SPIFFE SVID client
auth. The SDK keeps a thread-local connection per tenant context. Max connections are governed by tenant_class context and the tenant_class adoption model.

---

**Q8. Can a tenant run its own SQL via psql?**

No. Direct psql connections are refused — the proxy validates the client's SPIFFE SVID and Cedar permit set; raw connections
bypass both. The tenant connects through the SDK or via the `cloud-data` REST/gRPC API. There's a `oya data sql` CLI for ad-hoc
queries that handles auth correctly.

---

**Q9. How is geo-partitioning configured?**

```sql
ALTER TABLE orders PARTITION BY LIST (region);
CREATE PARTITION orders_us PARTITION OF orders FOR VALUES IN ('us') TABLESPACE = 'us-east-2';
CREATE PARTITION orders_eu PARTITION OF orders FOR VALUES IN ('eu') TABLESPACE = 'eu-west-1';
CREATE PARTITION orders_kr PARTITION OF orders FOR VALUES IN ('kr') TABLESPACE = 'ap-northeast-2';
```

Rows route to the partition based on the `region` column. Queries with `WHERE region = ...` are pruned to a single partition.
Cross-region queries are slower but supported (paid tenant_class).

---

**Q10. What's a "virtual cluster"?**

CockroachDB v25 virtual clusters provide multi-tenant logical isolation within a single physical cluster. Each tenant has its
own SQL endpoint, its own SQL gateway, its own resource pool. They share the storage layer but cannot see each other's data
(enforced by tenant-id row tagging + CRDB's tenant-scoped catalog).

---

**Q11. How does PITR (point-in-time recovery) work?**

CockroachDB streams WAL records to `cloud-storage` continuously. To restore:
1. Pick a target timestamp.
2. The restorer replays from the closest snapshot + WAL entries up to the target.
3. The restored data lands in a new database (you cannot overwrite the source DB without explicit Cedar permit).

PITR windows: demo_trial tenant_class 24 h, paid tenant_class 14 d, paid tenant_class 30 d, paid tenant_class 90 d hot + 7 y cold.

---

**Q12. Can I use Postgres extensions?**

Limited. CRDB supports a subset of Postgres extensions. We ship a curated set:
- **pgcrypto** (encrypted blobs)
- **uuid-ossp** (UUID generation; superseded by `gen_random_uuid()`)
- **pg_trgm** (trigram indexes)
- **postgis** (geo) — paid tenant_class
- **timescaledb-compat** (time-series functions; not the full TimescaleDB) — paid tenant_class

Adding an extension requires `cloud_data::Action::EnableExtension` + governance approval.

---

**Q13. How does the OLAP tier (ClickHouse) integrate with the OLTP tier?**

paid tenant_class ships a CDC pipeline: every CRDB change feed event is propagated to ClickHouse within ≤ 5 m (default). The OLAP tier
holds full historical state in columnar format; queries against ClickHouse are routed transparently for analytics workloads
(detected by query shape — large GROUP BY, no row-level filters).

---

**Q14. How does the ledger engine (TigerBeetle) integrate?**

TigerBeetle is a separate engine optimised for double-entry accounting with deterministic ordering. Use it for high-volume
financial transactions where strict ordering matters (e.g. `cloud-billing` payment processing). The SDK exposes
`oya_cloud_data_sdk::ledger` for TigerBeetle interactions.

---

**Q15. What's the graph engine for?**

Neo4j-5 at paid tenant_class for relationship-heavy workloads (org graphs, network graphs, knowledge graphs). The Cedar entity store
(used by `cloud-iam`) is itself stored as a graph for fast principal-resolution queries.

---

**Q16. How is read replica promotion (failover) handled?**

Cross-region read replicas are read-only by default. Promotion to leader requires:
1. Cedar permit `cloud_data::Action::PromoteReplica`.
2. Reviewer-agent thread (paid tenant_class).
3. Quorum check — the proposed new leader must have caught up to within 1 s of the current leader.
4. Promotion event anchored on `audit-chain`.

Forced promotion (`--force` flag) is allowed for break-glass but fires a P1 incident and writes a `cloud_data.emergency.replica_promote`
audit event.

---

**Q17. How is data residency enforced?**

At paid tenant_class, every tenant declares a data-residency policy:
```yaml
data_residency:
  primary: kr
  allowed_replicas: [kr]
  forbidden_replicas: ['*']
  cross_region_audit: pii-data-must-not-leave-kr
```

The Cedar policy `cloud_data::Action::Replicate` checks the destination region against the forbidden list. Failed replicas
refuse and audit-log.

---

**Q18. How is schema migration done?**

Schema migrations are versioned + applied via `oya data ddl apply --tenant <t> --migration <m>`. Each migration:
1. Pre-flight checks the schema against the tenant's existing schema.
2. Applies in a transaction (CRDB online schema change for index creation; offline for column drops).
3. Anchors the migration version on `audit-chain`.
4. Records the rollback path in the migration history.

CRDB's online schema changes support adding columns / indexes without blocking; dropping columns may require a maintenance window.

---

**Q19. Where does Foundry hook in?**

Foundry CI uses `cloud-data` as a tenant (`oyatie.foundry.<pipeline-id>`) for storing per-pipeline state (build cache, test
results, in-flight changeset metadata). Cedar permits for Foundry are narrow: `Connect`, `Read`, `Write` on its own DBs only;
no `PromoteReplica`, no `EmergencyDataExport`.

---

**Q20. How do I roll back a bad schema change?**

If the migration is reversible (no data loss):
```bash
./bin/oya data ddl rollback --tenant <t> --database <db> --to-version <n>
```

If the migration was destructive (DROP COLUMN, DROP TABLE), restore from PITR:
```bash
./bin/oya data pitr restore \
  --tenant <t> \
  --source-database <db> \
  --target-database <db>-rollback \
  --restore-to <pre-migration-ts>
```

Then re-promote the rollback as the live DB (a separate Cedar action that requires reviewer-agent at paid tenant_class).
