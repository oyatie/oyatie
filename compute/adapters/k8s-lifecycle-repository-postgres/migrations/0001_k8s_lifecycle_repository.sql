CREATE SCHEMA IF NOT EXISTS compute_k8s_lifecycle;

CREATE TABLE IF NOT EXISTS compute_k8s_lifecycle.clusters (
    tenant_id text NOT NULL CHECK (tenant_id <> ''),
    resource_id text NOT NULL CHECK (resource_id <> ''),
    desired_spec_json jsonb NOT NULL,
    cluster_json jsonb NOT NULL,
    observed_state text NOT NULL CHECK (observed_state <> ''),
    desired_state text NOT NULL CHECK (desired_state <> ''),
    schema_version integer NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, resource_id),
    CHECK (desired_spec_json ->> 'tenant_id' = tenant_id),
    CHECK (desired_spec_json ->> 'resource_id' = resource_id),
    CHECK (cluster_json ->> 'tenant_id' = tenant_id),
    CHECK (cluster_json ->> 'resource_id' = resource_id),
    CHECK (cluster_json ->> 'state' = observed_state),
    CHECK (cluster_json ->> 'desired_state' = desired_state)
);
CREATE INDEX IF NOT EXISTS clusters_reconciliation_scan
    ON compute_k8s_lifecycle.clusters (tenant_id, desired_state, updated_at, resource_id);
ALTER TABLE compute_k8s_lifecycle.clusters ENABLE ROW LEVEL SECURITY;
ALTER TABLE compute_k8s_lifecycle.clusters FORCE ROW LEVEL SECURITY;
CREATE POLICY clusters_tenant_isolation ON compute_k8s_lifecycle.clusters AS PERMISSIVE FOR ALL TO compute_k8s_lifecycle_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
CREATE POLICY clusters_require_tenant_guc ON compute_k8s_lifecycle.clusters AS RESTRICTIVE FOR ALL TO compute_k8s_lifecycle_runtime USING (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '') WITH CHECK (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '');

CREATE TABLE IF NOT EXISTS compute_k8s_lifecycle.operations (
    tenant_id text NOT NULL CHECK (tenant_id <> ''),
    principal_id text NOT NULL CHECK (principal_id <> ''),
    surface text NOT NULL CHECK (surface IN ('cloud.compute.k8s.cluster.create', 'cloud.compute.k8s.cluster.delete')),
    idempotency_key text NOT NULL CHECK (idempotency_key <> ''),
    resource_id text NOT NULL CHECK (resource_id <> ''),
    request_fingerprint text NOT NULL CHECK (request_fingerprint <> ''),
    receipt_kind text CHECK (receipt_kind IN ('create', 'delete')),
    receipt_json jsonb,
    schema_version integer NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz,
    PRIMARY KEY (tenant_id, principal_id, surface, idempotency_key),
    CHECK ((receipt_kind IS NULL) = (receipt_json IS NULL)),
    CHECK ((receipt_json IS NULL) = (completed_at IS NULL))
);
CREATE INDEX IF NOT EXISTS operations_resource_lookup
    ON compute_k8s_lifecycle.operations (tenant_id, resource_id, created_at);
ALTER TABLE compute_k8s_lifecycle.operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE compute_k8s_lifecycle.operations FORCE ROW LEVEL SECURITY;
CREATE POLICY operations_tenant_isolation ON compute_k8s_lifecycle.operations AS PERMISSIVE FOR ALL TO compute_k8s_lifecycle_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
CREATE POLICY operations_require_tenant_guc ON compute_k8s_lifecycle.operations AS RESTRICTIVE FOR ALL TO compute_k8s_lifecycle_runtime USING (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '') WITH CHECK (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '');

GRANT SELECT, INSERT, UPDATE ON compute_k8s_lifecycle.clusters TO compute_k8s_lifecycle_runtime;
GRANT SELECT, INSERT, UPDATE ON compute_k8s_lifecycle.operations TO compute_k8s_lifecycle_runtime;
