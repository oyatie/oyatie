use compute_k8s_api::{
    CloudComputeK8sAcceptanceApiError as Error, CloudComputeK8sLifecycleRepository,
    CloudComputeK8sLifecycleRepositoryError,
};
use compute_k8s_lifecycle_repository_postgres::PgK8sLifecycleRepository;
use sqlx::PgPool;

use super::fixtures::*;
use crate::support::{create_command, delete_command, tenant_count};

pub(super) async fn assert_compatibility(repository: &PgK8sLifecycleRepository, app: &PgPool) {
    let create = create_command("ten_alpha", "pending", "legacy-first");
    let original = repository.commit_create(create.clone()).await.unwrap();
    assert_eq!(
        accept(repository, pending_request("new", "legacy-first"))
            .await
            .unwrap_err(),
        Error::OperationContractMismatch
    );
    assert_eq!(
        read(repository, operation_read_request("legacy-first"))
            .await
            .unwrap_err(),
        Error::OperationContractMismatch
    );
    assert!(matches!(
        repository
            .commit_create(create_command("ten_alpha", "pending", "durable-key"))
            .await,
        Err(CloudComputeK8sLifecycleRepositoryError::IdempotencyKeyReused { .. })
    ));
    assert_eq!(
        repository.commit_create(create.clone()).await.unwrap(),
        original
    );
    let deletion = delete_command("ten_alpha", "pending", "durable-key");
    let deleted = repository.commit_deletion(deletion.clone()).await.unwrap();
    accept(repository, pending_request("after-delete", "after-delete"))
        .await
        .unwrap();
    assert_eq!(repository.commit_deletion(deletion).await.unwrap(), deleted);
    repository
        .commit_deletion(delete_command("ten_alpha", "pending", "delete-only"))
        .await
        .unwrap();
    assert_eq!(
        read(repository, operation_read_request("delete-only"))
            .await
            .unwrap(),
        compute_k8s_api::CloudComputeK8sOperationLookup::NotObserved
    );
    let same_key_different_surface =
        accept(repository, pending_request("create-surface", "delete-only"))
            .await
            .unwrap();
    assert_eq!(
        same_key_different_surface.operation.receipt.request_id,
        "create-surface"
    );
    assert_eq!(repository.commit_create(create).await.unwrap(), original);
    assert_eq!(
        tenant_count(
            app,
            "ten_alpha",
            "SELECT count(*)::bigint FROM compute_k8s_lifecycle.clusters"
        )
        .await,
        1
    );
    let mut wrong_resource = operation_read_request("durable-key");
    wrong_resource.path_cluster_id = crate::support::cluster_id("ten_alpha", "wrong");
    assert_eq!(
        read(repository, wrong_resource).await.unwrap_err(),
        Error::ResourceMismatch
    );
    let mut unauthorized = operation_read_request("durable-key");
    unauthorized.authorization.decision_id = "test-create-proof".into();
    let create_proof = verifier(
        &unauthorized.principal,
        "test-create-proof",
        compute_k8s_api::CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
    );
    assert!(matches!(
        compute_k8s_api::get_cloud_compute_k8s_operation_from_api_with_authorization_verifier(
            repository,
            unauthorized,
            &create_proof
        )
        .await,
        Err(Error::Boundary(_))
    ));
    assert!(matches!(
        compute_k8s_api::get_cloud_compute_k8s_operation_from_api(
            repository,
            operation_read_request("durable-key")
        )
        .await,
        Err(Error::Boundary(_))
    ));
}
