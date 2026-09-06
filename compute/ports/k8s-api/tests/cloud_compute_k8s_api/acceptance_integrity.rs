use compute_k8s_api::{
    CloudComputeK8sAcceptanceApiError, CloudComputeK8sAcceptanceRepositoryError,
    accept_cloud_compute_k8s_cluster_from_api_with_authorization_verifier,
    get_cloud_compute_k8s_operation_from_api_with_authorization_verifier,
};
use shared_resource_provider_contract_kernel::OperationState;

use super::acceptance_test_repository::*;

async fn accepted_repository() -> AcceptanceTestRepository {
    let repository = AcceptanceTestRepository::default();
    let request = pending_request("original", "key");
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

#[tokio::test]
async fn returned_receipt_identity_and_state_are_fail_closed() {
    for (mutation, expected) in [
        ("key", CloudComputeK8sAcceptanceApiError::IntegrityViolation),
        (
            "surface",
            CloudComputeK8sAcceptanceApiError::OperationContractMismatch,
        ),
        (
            "resource",
            CloudComputeK8sAcceptanceApiError::ResourceMismatch,
        ),
        (
            "tenant",
            CloudComputeK8sAcceptanceApiError::ResourceMismatch,
        ),
        (
            "request",
            CloudComputeK8sAcceptanceApiError::IntegrityViolation,
        ),
        (
            "timestamp",
            CloudComputeK8sAcceptanceApiError::IntegrityViolation,
        ),
        (
            "state",
            CloudComputeK8sAcceptanceApiError::IntegrityViolation,
        ),
    ] {
        let repository = accepted_repository().await;
        repository.mutate_snapshot(|snapshot| match mutation {
            "key" => snapshot.receipt.operation_key.idempotency_key = "other".into(),
            "surface" => snapshot.receipt.operation_key.surface = "other.surface".into(),
            "resource" => {
                snapshot.receipt.intent.resource_id =
                    "oyatie:cloud:region-home:ten_alpha:k8s:other".into()
            }
            "tenant" => snapshot.receipt.intent.tenant_id = "ten_other".into(),
            "request" => snapshot.receipt.request_id = " ".into(),
            "timestamp" => snapshot.receipt.accepted_at_epoch_seconds = 0,
            "state" => snapshot.state = OperationState::Running,
            _ => unreachable!(),
        });
        let request = read_request("read", "key");
        let verifier = verifier(
            &request.authorization.decision_id,
            compute_k8s_api::CLOUD_COMPUTE_K8S_OPERATION_GET_SURFACE,
        );
        let error = get_cloud_compute_k8s_operation_from_api_with_authorization_verifier(
            &repository,
            request,
            &verifier,
        )
        .await
        .unwrap_err();
        assert_eq!(error, expected, "{mutation}");
    }
}

#[tokio::test]
async fn repository_errors_map_without_disclosing_a_receipt() {
    let cases = [
        (
            CloudComputeK8sAcceptanceRepositoryError::IdempotencyKeyReused,
            CloudComputeK8sAcceptanceApiError::IdempotencyKeyReused,
            422,
        ),
        (
            CloudComputeK8sAcceptanceRepositoryError::OperationContractMismatch,
            CloudComputeK8sAcceptanceApiError::OperationContractMismatch,
            422,
        ),
        (
            CloudComputeK8sAcceptanceRepositoryError::ResourceMismatch,
            CloudComputeK8sAcceptanceApiError::ResourceMismatch,
            422,
        ),
        (
            CloudComputeK8sAcceptanceRepositoryError::Unavailable,
            CloudComputeK8sAcceptanceApiError::RepositoryUnavailable,
            503,
        ),
        (
            CloudComputeK8sAcceptanceRepositoryError::OutcomeUnknown,
            CloudComputeK8sAcceptanceApiError::OutcomeUnknown,
            503,
        ),
        (
            CloudComputeK8sAcceptanceRepositoryError::IntegrityViolation,
            CloudComputeK8sAcceptanceApiError::IntegrityViolation,
            503,
        ),
    ];
    for (repository_error, expected, status) in cases {
        let repository = AcceptanceTestRepository::default();
        repository.fail_accept_with(repository_error.clone());
        let request = pending_request("request", "key");
        let verifier = pending_create_verifier(&request);
        let error = accept_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
            &repository,
            request,
            &verifier,
        )
        .await
        .unwrap_err();
        assert_eq!(error, expected);
        assert_eq!(error.status_code(), status);
    }
}

#[tokio::test]
async fn read_repository_errors_use_the_same_fail_closed_mapping() {
    let repository = AcceptanceTestRepository::default();
    repository.fail_read_with(CloudComputeK8sAcceptanceRepositoryError::OutcomeUnknown);
    let request = read_request("request", "key");
    let verifier = verifier(
        &request.authorization.decision_id,
        compute_k8s_api::CLOUD_COMPUTE_K8S_OPERATION_GET_SURFACE,
    );
    assert_eq!(
        get_cloud_compute_k8s_operation_from_api_with_authorization_verifier(
            &repository,
            request,
            &verifier
        )
        .await
        .unwrap_err(),
        CloudComputeK8sAcceptanceApiError::OutcomeUnknown,
    );
}
