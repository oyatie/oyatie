# ADR-AN-003 — Row-Level Tenant Isolation (Defense-In-Depth)

**Scope:** analytics µservice only. Parent: ADR-0193 §"Multi-tenancy isolation".
**Status:** Accepted
**Date:** 2026-05-18
**Owner:** council-analytics + axis-compliance

## Context

The canonical isolation primitive (per ADR-0193) is **database-per-tenant** (`tenant_${tenant_id}`). This is a strong primary defense: tenant queries hit only their own database; cross-tenant access requires explicit `remote()` or `MERGE` cross-database, which is blocked by ClickHouse RBAC.

However, three classes of table legitimately need to be cross-tenant:

1. **Fleet-wide ops dashboards** — internal capacity-planning views aggregating across tenants.
2. **Billing finalization** — cross-tenant for the platform's accounts-receivable function.
3. **Compliance audit chain** — internal compliance officer reads the audit chain across tenants under explicit court order or regulator request.

For these tables, we need *row-level* isolation as a secondary defense.

## Decision

For any table that legitimately holds rows from multiple tenants, apply a ClickHouse row-level policy keyed to `tenant_id`:

```sql
CREATE ROW POLICY tenant_scope_policy ON fleet_internal.cross_tenant_table
USING tenant_id = currentUserTenantId()
TO ALL;
```

Where `currentUserTenantId()` is a UDF resolved from the ClickHouse user (which the adapter sets to `tenant_${tid}_reader` per IP-002).

For internal admin / compliance officer roles, the policy is relaxed:

```sql
CREATE ROW POLICY tenant_scope_policy_admin ON fleet_internal.cross_tenant_table
USING 1
TO Role::InternalAdmin, Role::InternalComplianceOfficer;
```

## Alternatives considered

1. **Database-per-tenant for cross-tenant tables.** Doesn't work — fleet rollups require seeing all tenants.
2. **View-based isolation** (`CREATE VIEW tenant_view AS SELECT * FROM cross_tenant WHERE tenant_id = X`). Rejected — easy to bypass at the SQL layer; not enforced by the engine.
3. **Per-tenant materialized projection** of the cross-tenant table. Rejected — duplicates storage for every tenant.

## Defense-in-depth stack

For the small set of cross-tenant tables, isolation is:

1. **Cedar policy** (gateway layer) — `principal.tenant_id == resource.tenant_id` unless `InternalAdmin`.
2. **Adapter-layer `assert_same_tenant`** (kernel) — before SQL dispatch, the adapter verifies the qualified table is in the principal's database. For cross-tenant tables (intentionally cross-tenant), this check is skipped (table is on an allow-list).
3. **ClickHouse RBAC** — the per-tenant user has GRANT on its own database only; cross-tenant tables are in a separate `fleet_internal` database that only `Role::InternalAdmin` can access directly.
4. **Row-level policy** — even if RBAC is bypassed, the row-level policy filters rows to the principal's tenant_id.
5. **Audit chain** — every query against `fleet_internal.*` emits an audit event with `(principal, justification_ref)`.

## Consequences

- **Positive.** Three independent layers protect cross-tenant data leak. Penetration test verifies all four layers.
- **Negative.** Marginal query overhead from row-policy evaluation (~5% per Cloudflare's ClickHouse R2 analytics blog).
- **Mitigation.** Only applied to the small set of cross-tenant tables; primary defense remains database-per-tenant.

## Rollout

- New cross-tenant tables: row-level policy at CREATE time, enforced via CI lane.
- Existing cross-tenant tables (if any, from prior fleet rollups): backfill row policies via reconciliation job.

## References

- ADR-0193 §"Multi-tenancy isolation".
- `microservices/analytics/threat-model.md` Spoofing + Information disclosure.
- `microservices/analytics/dpia.md` Risk 1 (cross-tenant data leak).
- ClickHouse row policy docs: https://clickhouse.com/docs/sql-reference/statements/create/row-policy
