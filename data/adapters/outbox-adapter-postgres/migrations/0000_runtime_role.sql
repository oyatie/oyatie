-- oya-data outbox runtime role contract (applied BEFORE 0001_outbox_events.sql).
--
-- The RLS policies in 0001 scope `TO oya_data_outbox_runtime` (NOT PUBLIC), so
-- that role MUST exist and hold the right privileges or the table is undefined
-- at runtime: a non-matching login role gets deny-all (outage) and a
-- BYPASSRLS/superuser role silently skips RLS entirely. This migration ships the
-- role contract IN the migration set so production's RLS posture is the EXACT
-- posture the live test exercises.
--
-- The runtime role:
--   * NOLOGIN — it is assumed (SET ROLE / GRANT membership) by a deploy-managed
--     login role, never logged into directly;
--   * NOBYPASSRLS — load-bearing: a BYPASSRLS role would silently skip every
--     policy in 0001, defeating tenant isolation (ADR-0567 D3);
--   * granted USAGE on the schema and SELECT, INSERT on the outbox table — the
--     CDC adapter only polls (SELECT) and producers only append (INSERT); no
--     UPDATE/DELETE is granted (outbox rows are immutable once written).
--
-- CREATE ROLE has no IF NOT EXISTS, so the create is guarded by a pg_roles
-- existence check (idempotent re-apply). The GRANTs are naturally idempotent.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'oya_data_outbox_runtime') THEN
        CREATE ROLE oya_data_outbox_runtime NOLOGIN NOBYPASSRLS;
    END IF;
END
$$;
CREATE SCHEMA IF NOT EXISTS oya_data_outbox;
GRANT USAGE ON SCHEMA oya_data_outbox TO oya_data_outbox_runtime;
