DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'compute_k8s_lifecycle_runtime') THEN
        CREATE ROLE compute_k8s_lifecycle_runtime NOLOGIN NOBYPASSRLS;
    END IF;
END
$$;
CREATE SCHEMA IF NOT EXISTS compute_k8s_lifecycle;
GRANT USAGE ON SCHEMA compute_k8s_lifecycle TO compute_k8s_lifecycle_runtime;
