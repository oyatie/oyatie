-- Tenant-lifecycle runtime role contract (applied BEFORE
-- 0001_tenant_lifecycle_store.sql).
--
-- The RLS policies in 0001 scope `TO tenancy_lifecycle_runtime` (NOT PUBLIC), so
-- that role MUST exist and hold the right privileges or the tables are undefined
-- at runtime: a non-matching login role gets deny-all (outage) and a
-- BYPASSRLS/superuser role silently skips RLS entirely. This migration ships the
-- role contract IN the migration set so production's RLS posture is the EXACT
-- posture the live test exercises (mirrors the oya-data outbox precedent,
-- ADR-0569 D3 / libs/oya-data-outbox-adapter-postgres/migrations/0000_runtime_role.sql).
--
-- The runtime role:
--   * NOLOGIN — it is assumed (SET ROLE / GRANT membership) by a deploy-managed
--     login role, never logged into directly;
--   * NOBYPASSRLS — load-bearing: a BYPASSRLS role would silently skip every
--     policy in 0001, defeating tenant isolation (ADR-0567 D3);
--   * granted USAGE on the schema; per-table SELECT/INSERT/UPDATE/DELETE grants
--     are issued in 0001 alongside the tables they target.
--
-- CREATE ROLE has no IF NOT EXISTS, so the create is guarded by a pg_roles
-- existence check (idempotent re-apply). The schema-create and GRANT are
-- naturally idempotent.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'tenancy_lifecycle_runtime') THEN
        CREATE ROLE tenancy_lifecycle_runtime NOLOGIN NOBYPASSRLS;
    END IF;
END
$$;
CREATE SCHEMA IF NOT EXISTS tenancy_lifecycle;
GRANT USAGE ON SCHEMA tenancy_lifecycle TO tenancy_lifecycle_runtime;
