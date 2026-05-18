---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-sheets-preview
phase: P01-sheets-foundation
impl_plan_id: IP-007-cell-grid-adapter-postgres-and-materialized-views
status: pending
owner: axis-sheets
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-citus-rls-enforced]
depends_on: [IP-002]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: cell-grid — adapter-postgres with materialized-view caches for hot ranges

## Intent

Author `oya-sheets-cell-grid-adapter-postgres`: workbook metadata + cell hot-tier rows + per-(workbook, sheet) materialized-view caches for frequently-accessed ranges. Citus partition + RLS enforced.

## ChangeSet boundary

One crate + Postgres schema migration:
- `oya-sheets-cell-grid-adapter-postgres`
- `microservices/sheets/src/migrations/V001__workbook_cell_schema.sql`

## Code Shape

Migration SQL (excerpt):

```sql
CREATE TABLE workbooks (
    tenant_id BYTEA NOT NULL,
    workbook_id TEXT PRIMARY KEY,
    version_sha TEXT NOT NULL,
    parent_version_sha TEXT,
    author_oidc_sub TEXT NOT NULL,
    title TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
ALTER TABLE workbooks ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON workbooks USING (tenant_id = current_setting('app.current_tenant_id')::bytea);
SELECT create_distributed_table('workbooks', 'tenant_id');

CREATE TABLE cells_hot (
    tenant_id BYTEA NOT NULL,
    workbook_id TEXT NOT NULL,
    sheet_id TEXT NOT NULL,
    cell_ref TEXT NOT NULL,
    value JSONB,
    formula TEXT,
    format_id TEXT,
    data_class TEXT NOT NULL,
    formula_error TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, workbook_id, sheet_id, cell_ref)
);
ALTER TABLE cells_hot ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON cells_hot USING (tenant_id = current_setting('app.current_tenant_id')::bytea);
SELECT create_distributed_table('cells_hot', 'tenant_id');

CREATE MATERIALIZED VIEW workbook_hot_range_cache AS
  SELECT tenant_id, workbook_id, sheet_id, jsonb_agg(value ORDER BY cell_ref) AS values
  FROM cells_hot
  GROUP BY tenant_id, workbook_id, sheet_id;
```

## Acceptance Gates

```bash
cargo check -p oya-sheets-cell-grid-adapter-postgres
cargo nextest run -p oya-sheets-cell-grid-adapter-postgres --test postgres_integration -- --include-ignored
cargo run -p oya-dev-cli -- gate validate citus-rls-enforced --microservice sheets
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_postgres_workbook_crud` | basic CRUD with RLS enforced |
| `test_citus_tenant_partition` | queries route to single shard by tenant_id |
| `test_rls_blocks_cross_tenant` | non-matching tenant context returns empty result |
| `test_materialized_view_refresh` | hot range cache stays current with cell edits |

## Halt Conditions

- RLS not enforced — STOP. T-I-01 mitigation foundational.
- Materialized view consistency drift — STOP.

## Next IP

[`IP-008-formatting-pivot-charts-data-validation.md`](IP-008-formatting-pivot-charts-data-validation.md)

## References

- PRD §"Bounded Contexts".
- threat-model.md T-I-01.
- Citus distributed-table docs.
- Postgres RLS docs.
