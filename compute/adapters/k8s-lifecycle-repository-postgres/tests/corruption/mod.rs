use compute_k8s_api::{
    CloudComputeK8sLifecycleRepository, CloudComputeK8sLifecycleRepositoryError,
};
use compute_k8s_lifecycle_repository_postgres::PgK8sLifecycleRepository;
use sqlx::PgPool;

use crate::support::{create_command, delete_command, tenant_count};

pub(crate) async fn assert_corruption_fails_closed(
    setup: &PgPool,
    app: &PgPool,
    repository: &PgK8sLifecycleRepository,
) {
    cluster_projection_corruption_rolls_back(setup, app, repository).await;
    receipt_corruption_refuses_replay(setup, app, repository).await;
}

async fn cluster_projection_corruption_rolls_back(
    setup: &PgPool,
    app: &PgPool,
    repository: &PgK8sLifecycleRepository,
) {
    repository
        .commit_create(create_command(
            "ten_alpha",
            "cluster-corrupt",
            "create-cluster-corrupt",
        ))
        .await
        .expect("corruption fixture create commits");
    sqlx::query(
        "ALTER TABLE compute_k8s_lifecycle.clusters DROP CONSTRAINT clusters_data_class_matches_desired",
    )
    .execute(setup)
    .await
    .expect("remove SQL guard to inject stored projection corruption");
    let corrupted = sqlx::query(
        r#"UPDATE compute_k8s_lifecycle.clusters SET cluster_json = jsonb_set(cluster_json, '{data_class}', '"PII_IDENTIFYING"'::jsonb) WHERE tenant_id = 'ten_alpha' AND resource_id = 'oyatie:cloud:region-home:ten_alpha:k8s:cluster-corrupt'"#,
    )
    .execute(setup)
    .await
    .expect("inject cluster projection corruption");
    assert_eq!(corrupted.rows_affected(), 1);

    let deletion = delete_command("ten_alpha", "cluster-corrupt", "delete-cluster-corrupt");
    assert_eq!(
        repository.commit_deletion(deletion.clone()).await,
        Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation)
    );
    assert_eq!(
        tenant_count(
            app,
            "ten_alpha",
            "SELECT count(*)::bigint FROM compute_k8s_lifecycle.operations WHERE idempotency_key = 'delete-cluster-corrupt'",
        )
        .await,
        0
    );

    sqlx::query(
        r#"UPDATE compute_k8s_lifecycle.clusters SET cluster_json = jsonb_set(cluster_json, '{data_class}', '"PUBLIC"'::jsonb) WHERE tenant_id = 'ten_alpha' AND resource_id = 'oyatie:cloud:region-home:ten_alpha:k8s:cluster-corrupt'"#,
    )
    .execute(setup)
    .await
    .expect("repair injected cluster corruption");
    sqlx::query(
        "ALTER TABLE compute_k8s_lifecycle.clusters ADD CONSTRAINT clusters_data_class_matches_desired CHECK (desired_spec_json ? 'data_class' AND cluster_json ? 'data_class' AND cluster_json -> 'data_class' = desired_spec_json -> 'data_class')",
    )
    .execute(setup)
    .await
    .expect("restore SQL projection guard");
    repository
        .commit_deletion(deletion)
        .await
        .expect("same delete operation succeeds after repair");
}

async fn receipt_corruption_refuses_replay(
    setup: &PgPool,
    app: &PgPool,
    repository: &PgK8sLifecycleRepository,
) {
    repository
        .commit_create(create_command(
            "ten_alpha",
            "receipt-corrupt",
            "create-receipt-corrupt",
        ))
        .await
        .expect("receipt corruption fixture create commits");
    let deletion = delete_command("ten_alpha", "receipt-corrupt", "delete-receipt-corrupt");
    repository
        .commit_deletion(deletion.clone())
        .await
        .expect("receipt corruption fixture delete commits");
    let corrupted = sqlx::query(
        r#"UPDATE compute_k8s_lifecycle.operations SET receipt_json = jsonb_set(receipt_json, '{cluster,data_class}', '"PII_IDENTIFYING"'::jsonb) WHERE tenant_id = 'ten_alpha' AND idempotency_key = 'delete-receipt-corrupt'"#,
    )
    .execute(setup)
    .await
    .expect("inject receipt corruption");
    assert_eq!(corrupted.rows_affected(), 1);

    assert_eq!(
        repository.commit_deletion(deletion.clone()).await,
        Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation)
    );
    assert_eq!(
        tenant_count(
            app,
            "ten_alpha",
            "SELECT count(*)::bigint FROM compute_k8s_lifecycle.clusters WHERE resource_id = 'oyatie:cloud:region-home:ten_alpha:k8s:receipt-corrupt' AND desired_state = 'deleted' AND cluster_json ->> 'data_class' = 'PUBLIC'",
        )
        .await,
        1
    );

    sqlx::query(
        r#"UPDATE compute_k8s_lifecycle.operations SET receipt_json = jsonb_set(receipt_json, '{cluster,data_class}', '"PUBLIC"'::jsonb) WHERE tenant_id = 'ten_alpha' AND idempotency_key = 'delete-receipt-corrupt'"#,
    )
    .execute(setup)
    .await
    .expect("repair injected receipt corruption");
    repository
        .commit_deletion(deletion)
        .await
        .expect("replay succeeds only after receipt repair");
}
