-- SCIM identity store: Postgres/RLS durable schema.
--
-- Tenant isolation is enforced by ENABLE + FORCE ROW LEVEL SECURITY under TWO
-- policies per table, keyed on the canonical session GUC
-- current_setting('oyatie.tenant_id', true) — the SAME GUC the shared command
-- kernel sets via `SELECT set_config('oyatie.tenant_id', $1, true)` before any
-- tenant-scoped statement (libs/shared-postgres-command-kernel
-- SET_LOCAL_TENANT_SQL):
--
--   1. A PERMISSIVE policy (FOR ALL) that ADMITS a row only when its tenant_id
--      equals the session GUC (USING + WITH CHECK). Postgres requires at least
--      one PERMISSIVE policy to admit a row to a forced-RLS table. With none,
--      the table is deny-all (no permissive grant), even for the row's own
--      tenant — that was the headline defect this migration corrects.
--   2. A RESTRICTIVE policy (FOR ALL) that hard-DENIES any access when the GUC
--      is unset or empty, so a missing per-tx SET_LOCAL_TENANT_SQL can never
--      fall back to an open scan. RESTRICTIVE intersects the permissive grant —
--      a row is visible only if the permissive policy admits it AND every
--      restrictive policy admits it.
--
-- The policy-subject role in every `TO <role>` clause below is
-- identity_scim_runtime — it MUST stay in lockstep with the `RUNTIME_ROLE`
-- constant in src/lib.rs (the boot-time RLS-enforceability guard checks
-- membership in exactly that role). Change both together.
--
-- The runtime role MUST NOT carry BYPASSRLS or RLS would be silently skipped.
-- A CHECK (tenant_id <> '') on every tenant_id column is defense-in-depth so a
-- blank tenant can never be persisted. userName uniqueness is per-tenant via
-- UNIQUE (tenant_id, user_name): two tenants may reuse a userName but one tenant
-- may not (the SCIM 409 Uniqueness contract). external_id is OPTIONAL in SCIM
-- (RFC 7643 section 3.1) — it is stored nullable and faithful, NULL when the
-- payload omits it, never coerced to an empty string.
CREATE SCHEMA IF NOT EXISTS identity_scim;

CREATE TABLE IF NOT EXISTS identity_scim.identity_scim_users (
    tenant_id text NOT NULL CHECK (tenant_id <> ''),
    scim_id text NOT NULL,
    user_name text NOT NULL,
    external_id text,
    active boolean NOT NULL,
    payload_json jsonb NOT NULL,
    schema_version integer NOT NULL,
    updated_at timestamptz NOT NULL,
    PRIMARY KEY (tenant_id, scim_id),
    UNIQUE (tenant_id, user_name)
);
ALTER TABLE identity_scim.identity_scim_users ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_scim.identity_scim_users FORCE ROW LEVEL SECURITY;
CREATE POLICY identity_scim_users_tenant_isolation ON identity_scim.identity_scim_users AS PERMISSIVE FOR ALL TO identity_scim_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
CREATE POLICY identity_scim_users_require_tenant_guc ON identity_scim.identity_scim_users AS RESTRICTIVE FOR ALL TO identity_scim_runtime USING (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '') WITH CHECK (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '');
COMMENT ON TABLE identity_scim.identity_scim_users IS 'SCIM identity user store; tenant-scoped under oyatie.tenant_id, per-tenant userName uniqueness.';

CREATE TABLE IF NOT EXISTS identity_scim.identity_scim_groups (
    tenant_id text NOT NULL CHECK (tenant_id <> ''),
    scim_id text NOT NULL,
    display_name text NOT NULL,
    payload_json jsonb NOT NULL,
    schema_version integer NOT NULL,
    updated_at timestamptz NOT NULL,
    PRIMARY KEY (tenant_id, scim_id)
);
ALTER TABLE identity_scim.identity_scim_groups ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_scim.identity_scim_groups FORCE ROW LEVEL SECURITY;
CREATE POLICY identity_scim_groups_tenant_isolation ON identity_scim.identity_scim_groups AS PERMISSIVE FOR ALL TO identity_scim_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
CREATE POLICY identity_scim_groups_require_tenant_guc ON identity_scim.identity_scim_groups AS RESTRICTIVE FOR ALL TO identity_scim_runtime USING (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '') WITH CHECK (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '');
COMMENT ON TABLE identity_scim.identity_scim_groups IS 'SCIM identity group store; tenant-scoped under oyatie.tenant_id.';

-- Table privileges for the RLS-subject runtime role provisioned by
-- 0000_runtime_role.sql (the role MUST already exist — 0000 is applied first).
-- The SCIM adapter performs the full tenant-scoped CRUD on both tables (create +
-- replace + delete users/groups, read + list), so all of
-- SELECT/INSERT/UPDATE/DELETE is granted. The policies above still confine every
-- row to the session-GUC tenant; these grants only admit the role to the
-- relation, never widen its RLS scope.
GRANT SELECT, INSERT, UPDATE, DELETE ON identity_scim.identity_scim_users TO identity_scim_runtime;
GRANT SELECT, INSERT, UPDATE, DELETE ON identity_scim.identity_scim_groups TO identity_scim_runtime;
