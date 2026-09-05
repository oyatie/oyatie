use compute_k8s_api::{
    CLOUD_COMPUTE_K8S_OPERATION_GET_SURFACE, CloudComputeK8sAcceptanceApiError,
    accept_cloud_compute_k8s_cluster_from_api_with_authorization_verifier,
    get_cloud_compute_k8s_operation_from_api_with_authorization_verifier,
};

use super::acceptance_test_repository::*;

async fn accepted_repository() -> AcceptanceTestRepository {
    let repository = AcceptanceTestRepository::default();
    let request = pending_request("original", "receipt-key");
    let verifier = pending_create_verifier(&request);
    accept_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        &repository,
        request,
        &verifier,
    )
    .await
    .unwrap();
    repository
}

async fn acceptance_retry_error(
    repository: &AcceptanceTestRepository,
) -> CloudComputeK8sAcceptanceApiError {
    let request = pending_request("retry", "receipt-key");
    let verifier = pending_create_verifier(&request);
    accept_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        repository, request, &verifier,
    )
    .await
    .unwrap_err()
}

async fn recovery_error(
    repository: &AcceptanceTestRepository,
) -> CloudComputeK8sAcceptanceApiError {
    let request = read_request("read", "receipt-key");
    let verifier = verifier(
        &request.authorization.decision_id,
        CLOUD_COMPUTE_K8S_OPERATION_GET_SURFACE,
    );
    get_cloud_compute_k8s_operation_from_api_with_authorization_verifier(
        repository, request, &verifier,
    )
    .await
    .unwrap_err()
}

#[tokio::test]
async fn canonically_reordered_retry_returns_the_original_receipt() {
    let repository = accepted_repository().await;
    let mut retry = pending_request("retry", "receipt-key");
    retry.body.node_pools.reverse();
    for pool in &mut retry.body.node_pools {
        pool.security_groups.reverse();
    }
    let verifier = pending_create_verifier(&retry);
    let replay = accept_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        &repository,
        retry,
        &verifier,
    )
    .await
    .unwrap();

    assert_eq!(replay.operation.receipt.request_id, "original");
    assert_eq!(
        replay.operation.receipt.accepted_at_epoch_seconds,
        ACCEPTED_AT
    );
}

#[tokio::test]
async fn acceptance_rejects_repository_returned_different_valid_intent() {
    let repository = accepted_repository().await;
    repository.mutate_snapshot(|snapshot| {
        snapshot.receipt.intent.control_plane_version = "v1.30.3-oyatie.1".into();
    });

    assert_eq!(
        acceptance_retry_error(&repository).await,
        CloudComputeK8sAcceptanceApiError::IdempotencyKeyReused,
    );
}

#[tokio::test]
async fn acceptance_and_recovery_reject_invalid_stored_intent() {
    let acceptance_repository = accepted_repository().await;
    acceptance_repository
        .mutate_snapshot(|snapshot| snapshot.receipt.intent.node_pools[0].max_nodes = 0);
    assert_eq!(
        acceptance_retry_error(&acceptance_repository).await,
        CloudComputeK8sAcceptanceApiError::IntegrityViolation,
    );

    let recovery_repository = accepted_repository().await;
    recovery_repository
        .mutate_snapshot(|snapshot| snapshot.receipt.intent.node_pools[0].max_nodes = 0);
    assert_eq!(
        recovery_error(&recovery_repository).await,
        CloudComputeK8sAcceptanceApiError::IntegrityViolation,
    );
}

#[tokio::test]
async fn acceptance_and_recovery_reject_each_operation_key_identity_corruption() {
    for field in ["tenant", "principal"] {
        let acceptance_repository = accepted_repository().await;
        acceptance_repository.mutate_snapshot(|snapshot| match field {
            "tenant" => snapshot.receipt.operation_key.tenant_id = "ten_other".into(),
            "principal" => snapshot.receipt.operation_key.principal_id = "sp_other".into(),
            _ => unreachable!(),
        });
        assert_eq!(
            acceptance_retry_error(&acceptance_repository).await,
            CloudComputeK8sAcceptanceApiError::IntegrityViolation,
            "acceptance/{field}",
        );

        let recovery_repository = accepted_repository().await;
        recovery_repository.mutate_snapshot(|snapshot| match field {
            "tenant" => snapshot.receipt.operation_key.tenant_id = "ten_other".into(),
            "principal" => snapshot.receipt.operation_key.principal_id = "sp_other".into(),
            _ => unreachable!(),
        });
        assert_eq!(
            recovery_error(&recovery_repository).await,
            CloudComputeK8sAcceptanceApiError::IntegrityViolation,
            "recovery/{field}",
        );
    }
}
