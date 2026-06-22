-- Tenant lifecycle store: Postgres/RLS durable schema.
--
-- Tenant isolation is enforced by ENABLE + FORCE ROW LEVEL SECURITY under TWO
-- policies per table, keyed on the canonical session GUC
-- current_setting('oyatie.tenant_id', true) — the SAME GUC the shared command
-- kernel sets via `SELECT set_config('oyatie.tenant_id', $1, true)` before any
-- tenant-scoped statement (libs/oya-shared-postgres-command-kernel
-- SET_LOCAL_TENANT_SQL):
--
--   1. A PERMISSIVE policy (FOR ALL) that ADMITS a row only when its tenant_id
--      equals the session GUC (USING + WITH CHECK). Postgres requires at least
--      one PERMISSIVE policy to admit a row to a forced-RLS table. With none,
--      the table is deny-all (no permissive grant), even for the row's own
--      tenant — that was the headline defect this migration corrects.
--   2. A RESTRICTIVE policy (FOR ALL) that hard-DENIES any access when the GUC
--      is unset or empty (a missing per-tx SET_LOCAL_TENANT_SQL must never fall
--      back to an open scan). RESTRICTIVE policies intersect with the permissive
--      grant — a row is visible only if the permissive policy admits it AND
--      every restrictive policy admits it.
--
-- The runtime role MUST NOT carry BYPASSRLS or RLS would be silently skipped.
-- The role name used in every `TO <role>` clause below is "tenancy_lifecycle_runtime";
-- this name is mirrored in the adapter's RUNTIME_ROLE const
-- (tenancy/adapters/tenant-lifecycle-store-postgres/src/lib.rs) — change both together.
-- A CHECK (tenant_id <> '') on every tenant_id column is defense-in-depth so a
-- blank tenant can never be persisted (and the empty-GUC restrictive guard can
-- never be satisfied by a blank stored value). Idempotency-key replay is a
-- no-op via ON CONFLICT (tenant_id, idempotency_key) DO NOTHING on the
-- applied-writes table.
CREATE SCHEMA IF NOT EXISTS tenancy_lifecycle;

CREATE TABLE IF NOT EXISTS tenancy_lifecycle.tenancy_lifecycle_tenants (
    tenant_id text NOT NULL CHECK (tenant_id <> ''),
    resource_name text NOT NULL,
    display_name text NOT NULL,
    lifecycle_state text NOT NULL,
    payload_json jsonb NOT NULL,
    schema_version integer NOT NULL,
    updated_at timestamptz NOT NULL,
    PRIMARY KEY (tenant_id, resource_name)
);
ALTER TABLE tenancy_lifecycle.tenancy_lifecycle_tenants ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenancy_lifecycle.tenancy_lifecycle_tenants FORCE ROW LEVEL SECURITY;
CREATE POLICY tenancy_lifecycle_tenants_tenant_isolation ON tenancy_lifecycle.tenancy_lifecycle_tenants AS PERMISSIVE FOR ALL TO tenancy_lifecycle_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
CREATE POLICY tenancy_lifecycle_tenants_require_tenant_guc ON tenancy_lifecycle.tenancy_lifecycle_tenants AS RESTRICTIVE FOR ALL TO tenancy_lifecycle_runtime USING (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '') WITH CHECK (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '');
COMMENT ON TABLE tenancy_lifecycle.tenancy_lifecycle_tenants IS 'Tenant lifecycle Postgres/RLS tenant aggregate store; tenant-scoped under oyatie.tenant_id.';

CREATE TABLE IF NOT EXISTS tenancy_lifecycle.tenancy_lifecycle_applied_writes (
    tenant_id text NOT NULL CHECK (tenant_id <> ''),
    idempotency_key text NOT NULL,
    payload_json jsonb NOT NULL,
    schema_version integer NOT NULL,
    created_at timestamptz NOT NULL,
    PRIMARY KEY (tenant_id, idempotency_key)
);
ALTER TABLE tenancy_lifecycle.tenancy_lifecycle_applied_writes ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenancy_lifecycle.tenancy_lifecycle_applied_writes FORCE ROW LEVEL SECURITY;
CREATE POLICY tenancy_lifecycle_applied_writes_tenant_isolation ON tenancy_lifecycle.tenancy_lifecycle_applied_writes AS PERMISSIVE FOR ALL TO tenancy_lifecycle_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
CREATE POLICY tenancy_lifecycle_applied_writes_require_tenant_guc ON tenancy_lifecycle.tenancy_lifecycle_applied_writes AS RESTRICTIVE FOR ALL TO tenancy_lifecycle_runtime USING (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '') WITH CHECK (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '');
COMMENT ON TABLE tenancy_lifecycle.tenancy_lifecycle_applied_writes IS 'Tenant lifecycle idempotency dedup table; tenant-scoped under oyatie.tenant_id.';

CREATE TABLE IF NOT EXISTS tenancy_lifecycle.tenancy_lifecycle_operations (
    tenant_id text NOT NULL CHECK (tenant_id <> ''),
    operation_name text NOT NULL,
    operation_seq bigint NOT NULL,
    payload_json jsonb NOT NULL,
    schema_version integer NOT NULL,
    created_at timestamptz NOT NULL,
    PRIMARY KEY (tenant_id, operation_name)
);
ALTER TABLE tenancy_lifecycle.tenancy_lifecycle_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenancy_lifecycle.tenancy_lifecycle_operations FORCE ROW LEVEL SECURITY;
CREATE POLICY tenancy_lifecycle_operations_tenant_isolation ON tenancy_lifecycle.tenancy_lifecycle_operations AS PERMISSIVE FOR ALL TO tenancy_lifecycle_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
CREATE POLICY tenancy_lifecycle_operations_require_tenant_guc ON tenancy_lifecycle.tenancy_lifecycle_operations AS RESTRICTIVE FOR ALL TO tenancy_lifecycle_runtime USING (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '') WITH CHECK (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '');
COMMENT ON TABLE tenancy_lifecycle.tenancy_lifecycle_operations IS 'Tenant lifecycle AIP-151 operation ledger; tenant-scoped under oyatie.tenant_id.';
