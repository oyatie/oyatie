-- Tenant lifecycle store: Postgres/RLS durable schema.
--
-- Tenant isolation is enforced by ENABLE + FORCE ROW LEVEL SECURITY under a
-- RESTRICTIVE policy keyed on the canonical session GUC
-- current_setting('oyatie.tenant_id', true) — the SAME GUC the shared command
-- kernel sets via `SELECT set_config('oyatie.tenant_id', $1, true)` before any
-- tenant-scoped statement (libs/oya-shared-postgres-command-kernel
-- SET_LOCAL_TENANT_SQL). The runtime role MUST NOT carry BYPASSRLS or RLS would
-- be silently skipped. Idempotency-key replay is a no-op via
-- ON CONFLICT (tenant_id, idempotency_key) DO NOTHING on the applied-writes
-- table.
CREATE SCHEMA IF NOT EXISTS tenancy_lifecycle;

CREATE TABLE IF NOT EXISTS tenancy_lifecycle.tenancy_lifecycle_tenants (
    tenant_id text NOT NULL,
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
CREATE POLICY tenancy_lifecycle_tenants_tenant_rls ON tenancy_lifecycle.tenancy_lifecycle_tenants AS RESTRICTIVE FOR ALL TO tenancy_lifecycle_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
COMMENT ON TABLE tenancy_lifecycle.tenancy_lifecycle_tenants IS 'Tenant lifecycle Postgres/RLS tenant aggregate store; tenant-scoped under oyatie.tenant_id.';

CREATE TABLE IF NOT EXISTS tenancy_lifecycle.tenancy_lifecycle_applied_writes (
    tenant_id text NOT NULL,
    idempotency_key text NOT NULL,
    payload_json jsonb NOT NULL,
    schema_version integer NOT NULL,
    created_at timestamptz NOT NULL,
    PRIMARY KEY (tenant_id, idempotency_key)
);
ALTER TABLE tenancy_lifecycle.tenancy_lifecycle_applied_writes ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenancy_lifecycle.tenancy_lifecycle_applied_writes FORCE ROW LEVEL SECURITY;
CREATE POLICY tenancy_lifecycle_applied_writes_tenant_rls ON tenancy_lifecycle.tenancy_lifecycle_applied_writes AS RESTRICTIVE FOR ALL TO tenancy_lifecycle_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
COMMENT ON TABLE tenancy_lifecycle.tenancy_lifecycle_applied_writes IS 'Tenant lifecycle idempotency dedup table; tenant-scoped under oyatie.tenant_id.';

CREATE TABLE IF NOT EXISTS tenancy_lifecycle.tenancy_lifecycle_operations (
    tenant_id text NOT NULL,
    operation_name text NOT NULL,
    operation_seq bigint NOT NULL,
    payload_json jsonb NOT NULL,
    schema_version integer NOT NULL,
    created_at timestamptz NOT NULL,
    PRIMARY KEY (tenant_id, operation_name)
);
ALTER TABLE tenancy_lifecycle.tenancy_lifecycle_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenancy_lifecycle.tenancy_lifecycle_operations FORCE ROW LEVEL SECURITY;
CREATE POLICY tenancy_lifecycle_operations_tenant_rls ON tenancy_lifecycle.tenancy_lifecycle_operations AS RESTRICTIVE FOR ALL TO tenancy_lifecycle_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
COMMENT ON TABLE tenancy_lifecycle.tenancy_lifecycle_operations IS 'Tenant lifecycle AIP-151 operation ledger; tenant-scoped under oyatie.tenant_id.';
