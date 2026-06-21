-- SCIM identity store: Postgres/RLS durable schema.
--
-- Tenant isolation is enforced by ENABLE + FORCE ROW LEVEL SECURITY under a
-- RESTRICTIVE policy keyed on the canonical session GUC
-- current_setting('oyatie.tenant_id', true) — the SAME GUC the shared command
-- kernel sets via `SELECT set_config('oyatie.tenant_id', $1, true)` before any
-- tenant-scoped statement (libs/oya-shared-postgres-command-kernel
-- SET_LOCAL_TENANT_SQL). The runtime role MUST NOT carry BYPASSRLS or RLS would
-- be silently skipped. userName uniqueness is per-tenant via
-- UNIQUE (tenant_id, user_name): two tenants may reuse a userName but one tenant
-- may not (the SCIM 409 Uniqueness contract).
CREATE SCHEMA IF NOT EXISTS identity_scim;

CREATE TABLE IF NOT EXISTS identity_scim.identity_scim_users (
    tenant_id text NOT NULL,
    scim_id text NOT NULL,
    user_name text NOT NULL,
    external_id text NOT NULL,
    active boolean NOT NULL,
    payload_json jsonb NOT NULL,
    schema_version integer NOT NULL,
    updated_at timestamptz NOT NULL,
    PRIMARY KEY (tenant_id, scim_id),
    UNIQUE (tenant_id, user_name)
);
ALTER TABLE identity_scim.identity_scim_users ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_scim.identity_scim_users FORCE ROW LEVEL SECURITY;
CREATE POLICY identity_scim_users_tenant_rls ON identity_scim.identity_scim_users AS RESTRICTIVE FOR ALL TO identity_scim_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
COMMENT ON TABLE identity_scim.identity_scim_users IS 'SCIM identity user store; tenant-scoped under oyatie.tenant_id, per-tenant userName uniqueness.';

CREATE TABLE IF NOT EXISTS identity_scim.identity_scim_groups (
    tenant_id text NOT NULL,
    scim_id text NOT NULL,
    display_name text NOT NULL,
    payload_json jsonb NOT NULL,
    schema_version integer NOT NULL,
    updated_at timestamptz NOT NULL,
    PRIMARY KEY (tenant_id, scim_id)
);
ALTER TABLE identity_scim.identity_scim_groups ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_scim.identity_scim_groups FORCE ROW LEVEL SECURITY;
CREATE POLICY identity_scim_groups_tenant_rls ON identity_scim.identity_scim_groups AS RESTRICTIVE FOR ALL TO identity_scim_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
COMMENT ON TABLE identity_scim.identity_scim_groups IS 'SCIM identity group store; tenant-scoped under oyatie.tenant_id.';
