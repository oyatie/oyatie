/// Creates the non-login, non-bypass runtime role and grants schema usage.
pub const K8S_LIFECYCLE_RUNTIME_ROLE_MIGRATION: &str = r#"DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'compute_k8s_lifecycle_runtime') THEN
        CREATE ROLE compute_k8s_lifecycle_runtime NOLOGIN NOBYPASSRLS;
    END IF;
END
$$;
CREATE SCHEMA IF NOT EXISTS compute_k8s_lifecycle;
GRANT USAGE ON SCHEMA compute_k8s_lifecycle TO compute_k8s_lifecycle_runtime;
"#;

/// Creates the lifecycle tables, indexes, forced RLS policies, and runtime grants.
pub const K8S_LIFECYCLE_REPOSITORY_MIGRATION: &str = r#"CREATE SCHEMA IF NOT EXISTS compute_k8s_lifecycle;

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
    CONSTRAINT clusters_database_schema_version CHECK (schema_version = 1),
    CONSTRAINT clusters_json_objects CHECK (jsonb_typeof(desired_spec_json) = 'object' AND jsonb_typeof(cluster_json) = 'object'),
    CONSTRAINT clusters_desired_tenant_matches CHECK (desired_spec_json ? 'tenant_id' AND desired_spec_json ->> 'tenant_id' = tenant_id),
    CONSTRAINT clusters_desired_resource_matches CHECK (desired_spec_json ? 'resource_id' AND desired_spec_json ->> 'resource_id' = resource_id),
    CONSTRAINT clusters_record_tenant_matches CHECK (cluster_json ? 'tenant_id' AND cluster_json ->> 'tenant_id' = tenant_id),
    CONSTRAINT clusters_record_resource_matches CHECK (cluster_json ? 'resource_id' AND cluster_json ->> 'resource_id' = resource_id),
    CONSTRAINT clusters_region_matches_desired CHECK (desired_spec_json ? 'region' AND cluster_json ? 'region' AND cluster_json -> 'region' = desired_spec_json -> 'region'),
    CONSTRAINT clusters_flavor_matches_desired CHECK (desired_spec_json ? 'flavor' AND cluster_json ? 'flavor' AND cluster_json -> 'flavor' = desired_spec_json -> 'flavor'),
    CONSTRAINT clusters_version_matches_desired CHECK (desired_spec_json ? 'control_plane_version' AND cluster_json ? 'control_plane_version' AND cluster_json -> 'control_plane_version' = desired_spec_json -> 'control_plane_version'),
    CONSTRAINT clusters_privacy_matches_desired CHECK (desired_spec_json ? 'control_plane_private' AND cluster_json ? 'control_plane_private' AND cluster_json -> 'control_plane_private' = desired_spec_json -> 'control_plane_private'),
    CONSTRAINT clusters_node_count_matches_desired CHECK (desired_spec_json ? 'node_pools' AND jsonb_typeof(desired_spec_json -> 'node_pools') = 'array' AND cluster_json ? 'node_pool_count' AND cluster_json -> 'node_pool_count' = to_jsonb(jsonb_array_length(desired_spec_json -> 'node_pools'))),
    CONSTRAINT clusters_residency_matches_desired CHECK (desired_spec_json ? 'residency' AND cluster_json ? 'residency' AND cluster_json -> 'residency' = desired_spec_json -> 'residency'),
    CONSTRAINT clusters_data_class_matches_desired CHECK (desired_spec_json ? 'data_class' AND cluster_json ? 'data_class' AND cluster_json -> 'data_class' = desired_spec_json -> 'data_class'),
    CONSTRAINT clusters_created_at_matches_desired CHECK (desired_spec_json ? 'created_at_epoch_seconds' AND cluster_json ? 'created_at_epoch_seconds' AND cluster_json -> 'created_at_epoch_seconds' = desired_spec_json -> 'created_at_epoch_seconds'),
    CONSTRAINT clusters_record_schema_version CHECK (cluster_json ? 'schema_version' AND cluster_json -> 'schema_version' = '2'::jsonb),
    CONSTRAINT clusters_observed_state_matches CHECK (cluster_json ? 'state' AND cluster_json ->> 'state' = observed_state AND observed_state IN ('creating', 'ready', 'reconciling', 'draining', 'deleted')),
    CONSTRAINT clusters_desired_state_matches CHECK (cluster_json ? 'desired_state' AND cluster_json ->> 'desired_state' = desired_state AND desired_state IN ('present', 'deleted'))
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
    receipt_digest text,
    schema_version integer NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz,
    PRIMARY KEY (tenant_id, principal_id, surface, idempotency_key),
    CONSTRAINT operations_database_schema_version CHECK (schema_version = 1),
    CONSTRAINT operations_receipt_atomicity CHECK ((receipt_kind IS NULL) = (receipt_json IS NULL) AND (receipt_json IS NULL) = (receipt_digest IS NULL) AND (receipt_digest IS NULL) = (completed_at IS NULL)),
    CONSTRAINT operations_receipt_digest_shape CHECK (receipt_digest IS NULL OR receipt_digest ~ '^[0-9a-f]{64}$'),
    CONSTRAINT operations_receipt_surface_matches CHECK (receipt_kind IS NULL OR (surface = 'cloud.compute.k8s.cluster.create' AND receipt_kind = 'create') OR (surface = 'cloud.compute.k8s.cluster.delete' AND receipt_kind = 'delete'))
);
CREATE INDEX IF NOT EXISTS operations_resource_lookup
    ON compute_k8s_lifecycle.operations (tenant_id, resource_id, created_at);
ALTER TABLE compute_k8s_lifecycle.operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE compute_k8s_lifecycle.operations FORCE ROW LEVEL SECURITY;
CREATE POLICY operations_tenant_isolation ON compute_k8s_lifecycle.operations AS PERMISSIVE FOR ALL TO compute_k8s_lifecycle_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
CREATE POLICY operations_require_tenant_guc ON compute_k8s_lifecycle.operations AS RESTRICTIVE FOR ALL TO compute_k8s_lifecycle_runtime USING (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '') WITH CHECK (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '');

GRANT SELECT, INSERT, UPDATE ON compute_k8s_lifecycle.clusters TO compute_k8s_lifecycle_runtime;
GRANT SELECT, INSERT, UPDATE ON compute_k8s_lifecycle.operations TO compute_k8s_lifecycle_runtime;
"#;
