#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use compute_k8s_api::{
    CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE, CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
    CloudComputeK8sClusterCreateRequest, CloudComputeK8sClusterRecord,
    CloudComputeK8sCreateCommand, CloudComputeK8sDeleteCommand, CloudComputeK8sLifecycleRepository,
    CloudComputeK8sLifecycleRepositoryError, CloudComputeK8sNodePoolCreateRequest,
    CloudComputeK8sNodePoolFlavorSpec, CloudComputeK8sOperationKey, CloudComputeK8sQuotaEnvelope,
    CloudComputeK8sSecurityGroupRef,
};
use compute_k8s_lifecycle_repository_postgres::{
    CLUSTERS_TABLE, OPERATIONS_TABLE, PgK8sLifecycleConnectError, PgK8sLifecycleRepository,
    RUNTIME_ROLE, SCHEMA_NAME,
};
use shared_postgres_command_kernel::{SET_LOCAL_TENANT_SQL, split_migration_statements};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};

const ENABLE_ENV: &str = "OYATIE_BACKBONE_LIVE_POSTGRES";
const SETUP_URL_ENV: &str = "OYATIE_BACKBONE_POSTGRES_URL";
const APP_URL_ENV: &str = "OYATIE_BACKBONE_POSTGRES_APP_URL";

fn require_enabled() {
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

async fn pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(8)
        .connect(database_url)
        .await
        .expect("connect to disposable Postgres")
}

async fn current_role(pool: &PgPool) -> String {
    sqlx::query_scalar("SELECT current_user::text")
        .fetch_one(pool)
        .await
        .expect("read current role")
}

async fn setup_schema(setup: &PgPool, app_role: &str) {
    sqlx::query(&format!("DROP SCHEMA IF EXISTS {SCHEMA_NAME} CASCADE"))
        .execute(setup)
        .await
        .expect("drop prior test schema");
    for migration in [
        include_str!("../migrations/0000_runtime_role.sql"),
        include_str!("../migrations/0001_k8s_lifecycle_repository.sql"),
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

fn cluster_id(tenant_id: &str, name: &str) -> String {
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

fn create_command(tenant_id: &str, name: &str, key: &str) -> CloudComputeK8sCreateCommand {
    CloudComputeK8sCreateCommand {
        operation_key: operation_key(tenant_id, CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE, key),
        fingerprint: format!("fingerprint:{tenant_id}:{name}"),
        desired_spec: desired_spec(tenant_id, name),
        cluster: cluster_record(tenant_id, name),
        request_id: format!("request:create:{tenant_id}:{name}"),
    }
}

fn delete_command(tenant_id: &str, name: &str, key: &str) -> CloudComputeK8sDeleteCommand {
    CloudComputeK8sDeleteCommand {
        operation_key: operation_key(tenant_id, CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE, key),
        resource_id: compute_resource_id(&cluster_id(tenant_id, name)),
        request_id: format!("request:delete:{tenant_id}:{name}"),
    }
}

fn compute_resource_id(value: &str) -> compute_resource::ResourceId {
    compute_resource::ResourceId::new(value.to_string()).expect("test resource id is valid")
}

async fn tenant_count(pool: &PgPool, tenant_id: &str, sql: &str) -> i64 {
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

#[tokio::test]
#[ignore = "live postgres"]
async fn live_repository_is_durable_isolated_concurrent_and_atomic() {
    require_enabled();
    let setup_url = std::env::var(SETUP_URL_ENV).expect("setup URL is required");
    let app_url = std::env::var(APP_URL_ENV).expect("app URL is required");
    let setup = pool(&setup_url).await;
    let app = pool(&app_url).await;
    let app_role = current_role(&app).await;
    setup_schema(&setup, &app_role).await;

    let privileged = PgK8sLifecycleRepository::from_pool(setup.clone()).await;
    assert!(matches!(
        privileged,
        Err(PgK8sLifecycleConnectError::RlsUnenforceable { .. })
    ));

    let repository = PgK8sLifecycleRepository::from_pool(app.clone())
        .await
        .expect("runtime role and FORCE RLS pass boot guard");
    let create = create_command("ten_alpha", "durable", "create-durable");
    let first = repository
        .commit_create(create.clone())
        .await
        .expect("initial create commits");

    let reopened = PgK8sLifecycleRepository::connect(&app_url)
        .await
        .expect("reopened repository passes boot guard");
    let replay = reopened
        .commit_create(create.clone())
        .await
        .expect("create receipt replays after reopen");
    assert_eq!(replay, first);

    let mut reused = create;
    reused.fingerprint = "different-fingerprint".to_string();
    assert!(matches!(
        reopened.commit_create(reused).await,
        Err(CloudComputeK8sLifecycleRepositoryError::IdempotencyKeyReused { .. })
    ));

    let replica_a = PgK8sLifecycleRepository::connect(&app_url)
        .await
        .expect("replica A connects");
    let replica_b = PgK8sLifecycleRepository::connect(&app_url)
        .await
        .expect("replica B connects");
    let concurrent = create_command("ten_alpha", "concurrent", "create-concurrent");
    let (left, right) = tokio::join!(
        replica_a.commit_create(concurrent.clone()),
        replica_b.commit_create(concurrent)
    );
    assert_eq!(
        left.expect("replica A succeeds"),
        right.expect("replica B replays")
    );
    assert_eq!(
        tenant_count(
            &app,
            "ten_alpha",
            "SELECT count(*)::bigint FROM compute_k8s_lifecycle.clusters WHERE resource_id = 'oyatie:cloud:region-home:ten_alpha:k8s:concurrent'",
        )
        .await,
        1
    );

    sqlx::query(
        "CREATE OR REPLACE FUNCTION compute_k8s_lifecycle.reject_create_receipt() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'injected create receipt failure'; END $$",
    )
    .execute(&setup)
    .await
    .expect("install create failure function");
    sqlx::query(
        "CREATE TRIGGER reject_create_receipt AFTER UPDATE ON compute_k8s_lifecycle.operations FOR EACH ROW WHEN (NEW.resource_id = 'oyatie:cloud:region-home:ten_alpha:k8s:create-rollback' AND NEW.receipt_kind = 'create') EXECUTE FUNCTION compute_k8s_lifecycle.reject_create_receipt()",
    )
    .execute(&setup)
    .await
    .expect("install create failure trigger");
    let rollback_create =
        create_command("ten_alpha", "create-rollback", "create-rollback-operation");
    assert_eq!(
        repository.commit_create(rollback_create.clone()).await,
        Err(CloudComputeK8sLifecycleRepositoryError::Unavailable)
    );
    assert_eq!(
        tenant_count(
            &app,
            "ten_alpha",
            "SELECT count(*)::bigint FROM compute_k8s_lifecycle.clusters WHERE resource_id = 'oyatie:cloud:region-home:ten_alpha:k8s:create-rollback'",
        )
        .await,
        0
    );
    assert_eq!(
        tenant_count(
            &app,
            "ten_alpha",
            "SELECT count(*)::bigint FROM compute_k8s_lifecycle.operations WHERE idempotency_key = 'create-rollback-operation'",
        )
        .await,
        0
    );
    sqlx::query("DROP TRIGGER reject_create_receipt ON compute_k8s_lifecycle.operations")
        .execute(&setup)
        .await
        .expect("remove create failure trigger");
    sqlx::query("DROP FUNCTION compute_k8s_lifecycle.reject_create_receipt()")
        .execute(&setup)
        .await
        .expect("remove create failure function");
    repository
        .commit_create(rollback_create)
        .await
        .expect("same create operation succeeds after injected failure is removed");

    let deletion = delete_command("ten_alpha", "durable", "delete-durable");
    let deleted = reopened
        .commit_deletion(deletion.clone())
        .await
        .expect("delete intent commits");
    assert_eq!(deleted.cluster.state, "creating");
    assert_eq!(deleted.cluster.desired_state, "deleted");
    let delete_replay = PgK8sLifecycleRepository::connect(&app_url)
        .await
        .expect("delete replay replica connects")
        .commit_deletion(deletion)
        .await
        .expect("delete receipt replays after reopen");
    assert_eq!(delete_replay, deleted);

    repository
        .commit_create(create_command("ten_beta", "private", "create-beta"))
        .await
        .expect("tenant beta create commits");
    let mut alpha = app.begin().await.expect("begin alpha RLS probe");
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind("ten_alpha")
        .execute(&mut *alpha)
        .await
        .expect("set alpha tenant scope");
    let hidden: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*)::bigint FROM {CLUSTERS_TABLE} WHERE resource_id = 'oyatie:cloud:region-home:ten_beta:k8s:private'"
    ))
    .fetch_one(&mut *alpha)
    .await
    .expect("cross-tenant select is filtered");
    assert_eq!(hidden, 0);
    let cross_tenant_insert = sqlx::query(&format!(
        "INSERT INTO {OPERATIONS_TABLE} (tenant_id, principal_id, surface, idempotency_key, resource_id, request_fingerprint, schema_version) VALUES ('ten_beta', 'forged', 'cloud.compute.k8s.cluster.delete', 'forged', 'foreign', 'foreign', 1)"
    ))
    .execute(&mut *alpha)
    .await;
    assert!(cross_tenant_insert.is_err());
    alpha.rollback().await.expect("rollback RLS probe");

    let mut no_scope = app.begin().await.expect("begin unset-GUC probe");
    let no_scope_clusters: i64 =
        sqlx::query_scalar(&format!("SELECT count(*)::bigint FROM {CLUSTERS_TABLE}"))
            .fetch_one(&mut *no_scope)
            .await
            .expect("unset GUC read is safely filtered");
    assert_eq!(no_scope_clusters, 0);
    no_scope.rollback().await.expect("rollback unset-GUC probe");

    repository
        .commit_create(create_command("ten_alpha", "rollback", "create-rollback"))
        .await
        .expect("rollback fixture create commits");
    sqlx::query(
        "CREATE OR REPLACE FUNCTION compute_k8s_lifecycle.reject_rollback_delete() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'injected lifecycle rollback'; END $$",
    )
    .execute(&setup)
    .await
    .expect("install failure function");
    sqlx::query(
        "CREATE TRIGGER reject_rollback_delete AFTER UPDATE ON compute_k8s_lifecycle.clusters FOR EACH ROW WHEN (NEW.resource_id = 'oyatie:cloud:region-home:ten_alpha:k8s:rollback' AND NEW.desired_state = 'deleted') EXECUTE FUNCTION compute_k8s_lifecycle.reject_rollback_delete()",
    )
    .execute(&setup)
    .await
    .expect("install failure trigger");
    let rollback_delete = delete_command("ten_alpha", "rollback", "delete-rollback");
    assert_eq!(
        repository.commit_deletion(rollback_delete.clone()).await,
        Err(CloudComputeK8sLifecycleRepositoryError::Unavailable)
    );

    let mut verify = app.begin().await.expect("begin rollback verification");
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind("ten_alpha")
        .execute(&mut *verify)
        .await
        .expect("set rollback tenant scope");
    let desired_state: String = sqlx::query_scalar(
        "SELECT desired_state FROM compute_k8s_lifecycle.clusters WHERE resource_id = 'oyatie:cloud:region-home:ten_alpha:k8s:rollback'",
    )
    .fetch_one(&mut *verify)
    .await
    .expect("read rolled-back cluster");
    let operation_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM compute_k8s_lifecycle.operations WHERE idempotency_key = 'delete-rollback'",
    )
    .fetch_one(&mut *verify)
    .await
    .expect("read rolled-back operation count");
    verify.rollback().await.expect("rollback verification read");
    assert_eq!(desired_state, "present");
    assert_eq!(operation_count, 0);

    sqlx::query("DROP TRIGGER reject_rollback_delete ON compute_k8s_lifecycle.clusters")
        .execute(&setup)
        .await
        .expect("remove failure trigger");
    sqlx::query("DROP FUNCTION compute_k8s_lifecycle.reject_rollback_delete()")
        .execute(&setup)
        .await
        .expect("remove failure function");
    let retried = repository
        .commit_deletion(rollback_delete)
        .await
        .expect("same operation succeeds after injected failure is removed");
    assert_eq!(retried.cluster.desired_state, "deleted");

    let app_flags =
        sqlx::query("SELECT rolsuper, rolbypassrls FROM pg_roles WHERE rolname = current_user")
            .fetch_one(&app)
            .await
            .expect("read app role flags");
    assert!(!app_flags.try_get::<bool, _>("rolsuper").unwrap());
    assert!(!app_flags.try_get::<bool, _>("rolbypassrls").unwrap());
}
