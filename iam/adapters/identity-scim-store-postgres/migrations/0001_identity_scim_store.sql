-- SCIM identity store: Postgres/RLS durable schema (review-only).
--
-- This is the declarative migration the future durable adapter will apply; it
-- is NOT applied by this crate. It is the byte-for-byte expectation of
-- `render_scim_postgres_migration(scim_postgres_storage_plan())`. Tenant
-- isolation is enforced by ENABLE + FORCE ROW LEVEL SECURITY under a
-- RESTRICTIVE policy keyed on current_setting('app.tenant_id', true); the
-- runtime role MUST NOT carry BYPASSRLS. userName uniqueness is per-tenant via
-- UNIQUE (tenant_id, user_name): two tenants may reuse a userName but one
-- tenant may not (the SCIM 409 Uniqueness contract).
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
CREATE POLICY identity_scim_users_tenant_rls ON identity_scim.identity_scim_users AS RESTRICTIVE FOR ALL TO identity_scim_runtime USING (tenant_id = current_setting('app.tenant_id', true)) WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
COMMENT ON TABLE identity_scim.identity_scim_users IS 'SCIM identity review-only Postgres/RLS storage plan; migrations are not applied by this crate.';

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
CREATE POLICY identity_scim_groups_tenant_rls ON identity_scim.identity_scim_groups AS RESTRICTIVE FOR ALL TO identity_scim_runtime USING (tenant_id = current_setting('app.tenant_id', true)) WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
COMMENT ON TABLE identity_scim.identity_scim_groups IS 'SCIM identity review-only Postgres/RLS storage plan; migrations are not applied by this crate.';
