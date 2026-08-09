---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: tasks
runbook_id: RB-custom-field-schema-migration
status: Accepted
date: 2026-05-17
owner_team: axis-tasks + ops-sre-reliability
severity_applicable: [Sev-2, Sev-3]
related_failure_modes: [FM-02]
related_dashboards: [throughput-and-engagement]
doc_status: published
---

# Runbook — Custom-Field Schema Migration Failure

## When this runbook fires

- `tasks_schema_migration_failure_rate > 0.1%` over 5 min, OR
- Tenant operator reports tasks erroring with `CustomFieldSchemaMismatch`, OR
- Migration job state machine emits `state=failed` for ≥ 1 tenant.

## Symptoms

- Reads error with `CustomFieldSchemaMismatch::Refused` for some tasks but not others (mid-migration).
- Tenant operator sees inconsistent custom-field rendering in views.
- Strict-coercion refusals (Hyrum #2) spike on legitimate writes.

## Probable causes

1. Schema-change initiated mid-flight; some tasks have new schema, some retain old.
2. Coercion failure on a specific row (type-incompatible legacy value).
3. Foreign-key reference to `CustomFieldSchema` row deleted before migration completes.

## Triage (within 30 min)

1. Acknowledge OnCall page.
2. Identify affected tenant + project + schema_id:
   ```promql
   tasks_schema_migration_failure_count{tenant_id_hashed!=""}
   ```
3. Pull migration job state:
   ```bash
   oya tasks schema-migration status --tenant <hashed-id> --project <project-id>
   ```
4. Identify failing rows (the per-row failure report in job state).

## Mitigation steps

### Step 1 — Rollback transactional partial-migration

```bash
oya tasks schema-migration rollback --job-id <id> --audit-reason "RB-custom-field-schema-migration"
```

This restores the prior schema_id for affected tasks; reads stop erroring.

### Step 2 — Fix the row-level type incompatibility

Surface failing rows to tenant operator:
```bash
oya tasks schema-migration failed-rows --job-id <id> --format csv > /tmp/failed-rows.csv
```

Tenant operator reviews + pre-casts the offending values, then re-runs migration.

### Step 3 — Re-run migration (idempotent)

```bash
oya tasks schema-migration apply --tenant <hashed-id> --project <project-id> --schema <new-schema-id> --audit-reason "RB-custom-field-schema-migration"
```

### Step 4 — If foreign-key dangling

```bash
oya tasks schema repair --project <project-id> --audit-reason "RB-custom-field-schema-migration"
```

## Recovery validation

| Metric | Target | After mitigation |
|---|---|---|
| `tasks_schema_migration_failure_rate` | < 0.1% | within 30 min |
| `tasks_custom_field_strict_coerce_refused_total` | baseline | within 30 min |
| Reads with `CustomFieldSchemaMismatch` errors | 0 | within 5 min |

## Post-incident review

- Was the schema-change pre-validated against existing rows?
- Should we add a pre-migration "type-incompatibility report" step?
- Update ADR-TASKS-0001 if a new coercion edge-case discovered.

## Drills

- Quarterly: simulated schema-change with known-incompatible row in synthetic tenant.

## References

- `failure-modes.md` FM-02.
- ADR-TASKS-0001 (data model + strict coercion).
- `dashboards/throughput-and-engagement.json`.
