use compute_k8s_api::{
    CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE, CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
    CloudComputeK8sClusterCreateRequest, CloudComputeK8sClusterRecord,
    CloudComputeK8sCreateCommand, CloudComputeK8sDeleteCommand,
    CloudComputeK8sNodePoolCreateRequest, CloudComputeK8sNodePoolFlavorSpec,
    CloudComputeK8sOperationKey, CloudComputeK8sQuotaEnvelope, CloudComputeK8sSecurityGroupRef,
};
use compute_k8s_lifecycle_repository_postgres::{
    K8S_LIFECYCLE_REPOSITORY_MIGRATION, K8S_LIFECYCLE_RUNTIME_ROLE_MIGRATION, RUNTIME_ROLE,
    SCHEMA_NAME,
};
use shared_postgres_command_kernel::{SET_LOCAL_TENANT_SQL, split_migration_statements};
use sqlx::{PgPool, postgres::PgPoolOptions};

pub(super) const ENABLE_ENV: &str = "OYATIE_BACKBONE_LIVE_POSTGRES";
pub(super) const SETUP_URL_ENV: &str = "OYATIE_BACKBONE_POSTGRES_URL";
pub(super) const APP_URL_ENV: &str = "OYATIE_BACKBONE_POSTGRES_APP_URL";

pub(super) fn require_enabled() {
    let enabled = std::env::var(ENABLE_ENV)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false);
    assert!(enabled, "live Postgres environment is not enabled");
}

pub(super) async fn pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(8)
        .connect(database_url)
        .await
        .expect("connect to disposable Postgres")
}

pub(super) async fn current_role(pool: &PgPool) -> String {
    sqlx::query_scalar("SELECT current_user::text")
        .fetch_one(pool)
        .await
        .expect("read current role")
}

pub(super) async fn setup_schema(setup: &PgPool, app_role: &str) {
    sqlx::query(&format!("DROP SCHEMA IF EXISTS {SCHEMA_NAME} CASCADE"))
        .execute(setup)
        .await
        .expect("drop prior test schema");
    for migration in [
        K8S_LIFECYCLE_RUNTIME_ROLE_MIGRATION,
        K8S_LIFECYCLE_REPOSITORY_MIGRATION,
    ] {
        for statement in split_migration_statements(migration) {
            sqlx::query(&statement)
                .execute(setup)
                .await
                .unwrap_or_else(|error| panic!("migration failed: {statement}\n{error}"));
        }
    }
    sqlx::query(&format!("GRANT {RUNTIME_ROLE} TO \"{app_role}\""))
        .execute(setup)
        .await
        .expect("grant runtime role membership");
}

pub(super) fn cluster_id(tenant_id: &str, name: &str) -> String {
    format!("oyatie:cloud:region-home:{tenant_id}:k8s:{name}")
}

fn desired_spec(tenant_id: &str, name: &str) -> CloudComputeK8sClusterCreateRequest {
    let resource_id = cluster_id(tenant_id, name);
    CloudComputeK8sClusterCreateRequest {
        resource_id,
        tenant_id: tenant_id.to_string(),
        region: "region-home".to_string(),
        flavor: "standard".to_string(),
        control_plane_version: "v1.30.2-oyatie.1".to_string(),
        control_plane_private: true,
        node_pools: vec![CloudComputeK8sNodePoolCreateRequest {
            id: "np-a".to_string(),
            az: "region-home-a".to_string(),
            cell_id: "cell-region-home-a-001".to_string(),
            subnet_id: format!("oyatie:cloud:region-home:{tenant_id}:subnet:prod-a"),
            security_groups: vec![CloudComputeK8sSecurityGroupRef {
                value: "sg-workload".to_string(),
                tenant_id: tenant_id.to_string(),
                region: "region-home".to_string(),
                subnet_id: format!("oyatie:cloud:region-home:{tenant_id}:subnet:prod-a"),
            }],
            flavor: CloudComputeK8sNodePoolFlavorSpec {
                class: "general_purpose".to_string(),
                vcpu: 4,
                memory_gb: 16,
                gpu_count: 0,
                local_ssd_gb: 100,
            },
            min_nodes: 1,
            max_nodes: 5,
            autoscaling_enabled: true,
        }],
        quota: CloudComputeK8sQuotaEnvelope {
            vcpu_limit: 128,
            memory_gb_limit: 512,
            gpu_limit: 8,
            local_ssd_gb_limit: 4_096,
            current_vcpu: 4,
            current_memory_gb: 16,
            current_gpu: 0,
            current_local_ssd_gb: 100,
        },
        residency: "strict_home_region".to_string(),
        data_class: "PUBLIC".to_string(),
        created_at_epoch_seconds: 1_700_100_010,
    }
}

fn cluster_record(tenant_id: &str, name: &str) -> CloudComputeK8sClusterRecord {
    CloudComputeK8sClusterRecord {
        resource_id: cluster_id(tenant_id, name),
        tenant_id: tenant_id.to_string(),
        region: "region-home".to_string(),
        flavor: "standard".to_string(),
        control_plane_version: "v1.30.2-oyatie.1".to_string(),
        control_plane_private: true,
        node_pool_count: 1,
        residency: "strict_home_region".to_string(),
        state: "creating".to_string(),
        desired_state: "present".to_string(),
        data_class: "PUBLIC".to_string(),
        created_at_epoch_seconds: 1_700_100_010,
        schema_version: 2,
    }
}

fn operation_key(tenant_id: &str, surface: &str, key: &str) -> CloudComputeK8sOperationKey {
    CloudComputeK8sOperationKey {
        tenant_id: tenant_id.to_string(),
        principal_id: "sp-compute-live".to_string(),
        surface: surface.to_string(),
        idempotency_key: key.to_string(),
    }
}

pub(super) fn create_command(
    tenant_id: &str,
    name: &str,
    key: &str,
) -> CloudComputeK8sCreateCommand {
    CloudComputeK8sCreateCommand {
        operation_key: operation_key(tenant_id, CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE, key),
        fingerprint: format!("fingerprint:{tenant_id}:{name}"),
        desired_spec: desired_spec(tenant_id, name),
        cluster: cluster_record(tenant_id, name),
        request_id: format!("request:create:{tenant_id}:{name}"),
    }
}

pub(super) fn delete_command(
    tenant_id: &str,
    name: &str,
    key: &str,
) -> CloudComputeK8sDeleteCommand {
    CloudComputeK8sDeleteCommand {
        operation_key: operation_key(tenant_id, CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE, key),
        resource_id: compute_resource_id(&cluster_id(tenant_id, name)),
        request_id: format!("request:delete:{tenant_id}:{name}"),
    }
}

fn compute_resource_id(value: &str) -> compute_resource::ResourceId {
    compute_resource::ResourceId::new(value.to_string()).expect("test resource id is valid")
}

pub(super) async fn tenant_count(pool: &PgPool, tenant_id: &str, sql: &str) -> i64 {
    let mut transaction = pool.begin().await.expect("begin tenant count");
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind(tenant_id)
        .execute(&mut *transaction)
        .await
        .expect("set tenant scope");
    let count = sqlx::query_scalar(sql)
        .fetch_one(&mut *transaction)
        .await
        .expect("count tenant rows");
    transaction
        .rollback()
        .await
        .expect("rollback read transaction");
    count
}
