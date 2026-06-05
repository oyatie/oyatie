---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-010-fleet-state-postgres-adapter
status: pending
execution_unit: ChangeSet
owner: axis-foundry-control-plane
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, oya-check-postgres-rls-enforced, oya-check-shardability]
depends_on: [IP-001, IP-004]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: fleet-state Postgres adapter (with RLS + tenant sharding)

## Intent

Postgres-backed implementation of `FleetStateRepository` port from IP-004; row-level-security; tenant-shard routing; PgBouncer pooled.

## Concrete File Targets

`…-agent-fleet-lifecycle-adapter-postgres/Cargo.toml` + `src/lib.rs` + `src/fleet_state_repo.rs` + `src/migrations/`.

Also: `…-capability-deployment-adapter-postgres/` (deployment history + capability definitions).

## Key code

```rust
// adapter-postgres/src/fleet_state_repo.rs
pub struct PostgresFleetStateRepo {
    pool: Arc<PgPool>,  // PgBouncer-pooled
}

#[async_trait]
impl FleetStateRepository for PostgresFleetStateRepo {
    async fn load(&self, tenant: &TenantId) -> Result<FleetState, RepositoryError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!("SET LOCAL app.tenant_id = $1", tenant.as_str())
            .execute(&mut *tx).await?;
        let rows = sqlx::query_as!(FleetStateRow, "SELECT * FROM fleet_state")
            .fetch_all(&mut *tx).await?;
        // RLS automatically scoped to current_setting('app.tenant_id')
        Ok(FleetState::from_rows(rows))
    }
    // ... drain / evict
}
```

## SQL migrations

```sql
-- migrations/0001_fleet_state.sql
CREATE TABLE fleet_state (
    tenant_id     TEXT NOT NULL,
    agent_id      UUID NOT NULL,
    capability_id TEXT NOT NULL,
    state         TEXT NOT NULL CHECK (state IN ('pending','healthy','draining','evicted')),
    healthy_count INTEGER,
    draining_count INTEGER,
    jurisdiction  TEXT NOT NULL CHECK (jurisdiction IN ('kr','eu','us','us-hc','jp','sg','au','in','br','ae','ksa')),
    pack          TEXT NOT NULL,
    data_class    TEXT NOT NULL,
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, agent_id)
) PARTITION BY HASH(tenant_id);

CREATE TABLE fleet_state_shard_0 PARTITION OF fleet_state FOR VALUES WITH (MODULUS 4, REMAINDER 0);
CREATE TABLE fleet_state_shard_1 PARTITION OF fleet_state FOR VALUES WITH (MODULUS 4, REMAINDER 1);
CREATE TABLE fleet_state_shard_2 PARTITION OF fleet_state FOR VALUES WITH (MODULUS 4, REMAINDER 2);
CREATE TABLE fleet_state_shard_3 PARTITION OF fleet_state FOR VALUES WITH (MODULUS 4, REMAINDER 3);

ALTER TABLE fleet_state ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON fleet_state
  USING (tenant_id = current_setting('app.tenant_id', true));
```

## Acceptance Gates

```bash
cargo check / build / clippy / nextest
buck2 build //:quality-lane-registry-authority-check # lane=postgres-rls-enforced --microservice foundry-supervisor
buck2 build //:quality-lane-registry-authority-check # lane=shardability --microservice foundry-supervisor
```

## Test Plan

| Test | Verifies |
|---|---|
| `tenant_isolation_at_query_level` | tenant-A query returns zero tenant-B rows |
| `rls_policy_active_on_every_table` | `pg_policies` query confirms |
| `shard_key_present` | LEAN check passes |
| `pgbouncer_session_local_isolated` | per-checkout `app.tenant_id` does not leak |

## Halt Conditions

- RLS missing on any tenant-scoped table.
- Shard key absent.

## Next IP

[`IP-011-rest-api.md`](IP-011-rest-api.md)

## References

- PostgreSQL RLS docs — `postgresql.org/docs/current/ddl-rowsecurity.html`.
- `policy/supervisor-isolation.md` TI-P-01..TI-P-06.
- PRD §"Horizontal Scalability".

## Wave 15 counterpart anchor

- Counterparts: Palantir AIP Operator, Azure AI Foundry deployments, and GitHub merge-queue controls.
- Gap closure: this IP closes fleet control, kill-switch propagation, and deployability evidence with tenant-scoped policy enforcement.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
