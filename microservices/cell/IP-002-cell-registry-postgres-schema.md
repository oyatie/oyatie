---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-cell-substrate
impl_plan_id: IP-002-cell-registry-postgres-schema
status: pending
owner: axis-cell-substrate
acceptance_lanes: [helm-lint, sqlx-prepare-check, oya-cell-rls-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: Postgres schema + per-pack shard topology + Helm chart

## Intent

Author the Postgres schema for cell-registry (tables: `cells`, `cell_assignments`, `migration_plans`, `hosts`, `cell_lifecycle_events`). Each cell-scoped table carries row-level-security (RLS) keyed on `(pack, cell_id)`. Helm chart deploys CloudNativePG-managed HA Postgres per pack.

## ChangeSet boundary

One ChangeSet: 1 Helm chart (oya-cell-postgres) + 1 schema migration `migrations/0001_cell_registry_initial.sql` + per-pack values.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cell/iac/helm/postgres/Chart.yaml` | create |
| `microservices/cell/iac/helm/postgres/values.yaml` | create (CloudNativePG operator config) |
| `microservices/cell/iac/helm/postgres/values-pack-kr.yaml` | create |
| `microservices/cell/migrations/0001_cell_registry_initial.sql` | create |
| `microservices/cell/migrations/0002_rls_policies.sql` | create |

## Schema Shape

```sql
-- cells table: authoritative cell metadata
CREATE TABLE cells (
  cell_id           VARCHAR(32) PRIMARY KEY,
  pack              VARCHAR(32) NOT NULL,
  region            VARCHAR(64) NOT NULL,
  state             VARCHAR(32) NOT NULL DEFAULT 'requested',
  cell_scope        VARCHAR(32) NOT NULL,
  capacity_envelope JSONB NOT NULL,
  version           VARCHAR(16) NOT NULL,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  decommissioned_at TIMESTAMPTZ NULL,
  signature         BYTEA NOT NULL  -- Ed25519 audit-chain seal
);

-- cell_assignments: (tenant_id, cell_id) binding
CREATE TABLE cell_assignments (
  tenant_id    VARCHAR(32) NOT NULL,
  cell_id      VARCHAR(32) NOT NULL REFERENCES cells(cell_id),
  pack         VARCHAR(32) NOT NULL,
  scope        VARCHAR(32) NOT NULL,  -- primary | ha-secondary | migrating-target | migrating-source
  assigned_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  released_at  TIMESTAMPTZ NULL,
  signature    BYTEA NOT NULL,
  PRIMARY KEY (tenant_id, scope),
  CONSTRAINT pack_match CHECK (
    pack = (SELECT pack FROM cells WHERE cells.cell_id = cell_assignments.cell_id)
  )
);

ALTER TABLE cell_assignments ENABLE ROW LEVEL SECURITY;
CREATE POLICY cell_assignments_pack_rls ON cell_assignments
  USING (pack = current_setting('app.session_pack')::varchar);

-- migration_plans
CREATE TABLE migration_plans (
  migration_id   VARCHAR(32) PRIMARY KEY,
  tenant_id      VARCHAR(32) NOT NULL,
  source_cell    VARCHAR(32) NOT NULL,
  target_cell    VARCHAR(32) NOT NULL,
  state          VARCHAR(32) NOT NULL DEFAULT 'planned',
  reason         VARCHAR(64) NOT NULL,
  started_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  completed_at   TIMESTAMPTZ NULL,
  checkpoint     JSONB,
  signature      BYTEA NOT NULL,
  UNIQUE (tenant_id, state) WHERE state IN ('planned','draining','copying','cutover')  -- advisory-lock effect
);

-- hosts
CREATE TABLE hosts (
  host_id         VARCHAR(32) PRIMARY KEY,
  pack            VARCHAR(32) NOT NULL,
  region          VARCHAR(64) NOT NULL,
  pool_state      VARCHAR(32) NOT NULL DEFAULT 'warm-standby',
  bound_cell_id   VARCHAR(32) NULL REFERENCES cells(cell_id),
  provisioned_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- cell_lifecycle_events: per-state-transition audit
CREATE TABLE cell_lifecycle_events (
  event_id        BIGSERIAL PRIMARY KEY,
  cell_id         VARCHAR(32) NOT NULL REFERENCES cells(cell_id),
  prev_state      VARCHAR(32),
  new_state       VARCHAR(32) NOT NULL,
  transitioned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  transitioned_by VARCHAR(255) NOT NULL,
  reason          TEXT,
  signature       BYTEA NOT NULL
);
```

## Acceptance Gates

```bash
helm lint microservices/cell/iac/helm/postgres
cargo run -p oya-dev-cli -- gate validate cell-rls-conformance
cargo sqlx prepare --check
```

## Test Plan

- Integration test: spin up Postgres in kind; apply migrations; insert sample rows; verify RLS denies cross-pack reads.
- Property test: every `cell_assignments` insert MUST satisfy `pack_match` constraint.

## Halt Conditions

- RLS policy missing on any cell-scoped table — fail lane.
- Cross-pack write attempt succeeds in integration test — block.

## Next IP

[`IP-003-cell-registry-kernel.md`](IP-003-cell-registry-kernel.md)

## References

- Bominal ADR-0009 + ADR-0019.
- CloudNativePG — `cloudnative-pg.io`.
- Postgres RLS — `postgresql.org/docs/current/ddl-rowsecurity.html`.
