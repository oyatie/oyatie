use compute_k8s_api::{
    CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE, CLOUD_COMPUTE_K8S_OPERATION_GET_SURFACE,
    CloudComputeK8sAcceptanceApiError, CloudComputeK8sOperationLookup,
    accept_cloud_compute_k8s_cluster_from_api,
    accept_cloud_compute_k8s_cluster_from_api_with_authorization_verifier,
    get_cloud_compute_k8s_operation_from_api,
    get_cloud_compute_k8s_operation_from_api_with_authorization_verifier,
};
use shared_resource_provider_contract_kernel::OperationState;

use super::acceptance_test_repository::*;

#[tokio::test]
async fn acceptance_replay_preserves_the_original_complete_receipt() {
    let repository = AcceptanceTestRepository::default();
    let original = pending_request("first-request", "accept-once");
    let verifier = pending_create_verifier(&original);
    let first = accept_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        &repository,
        original,
        &verifier,
    )
    .await
    .unwrap();
    let retry = pending_request("retry-request", "accept-once");
    let replay = accept_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        &repository,
        retry,
        &verifier,
    )
    .await
    .unwrap();

    assert_eq!(first.status_code(), 202);
    assert_eq!(replay.operation.receipt.request_id, "first-request");
    assert_eq!(
        replay.operation.receipt.accepted_at_epoch_seconds,
        ACCEPTED_AT
    );
    assert_eq!(replay.operation.state, OperationState::Accepted);
    let command = repository.last_command().unwrap();
    assert_eq!(
        command.operation_key.surface,
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE
    );
}

#[tokio::test]
async fn default_acceptance_and_read_refuse_before_repository_access() {
    let repository = AcceptanceTestRepository::default();
    let accept_error =
        accept_cloud_compute_k8s_cluster_from_api(&repository, pending_request("request", "key"))
            .await
            .unwrap_err();
    let read_error =
        get_cloud_compute_k8s_operation_from_api(&repository, read_request("request", "key"))
            .await
            .unwrap_err();

    assert!(matches!(
        accept_error,
        CloudComputeK8sAcceptanceApiError::Boundary(_)
    ));
    assert!(matches!(
        read_error,
        CloudComputeK8sAcceptanceApiError::Boundary(_)
    ));
    assert_eq!(repository.calls(), (0, 0));
}

#[tokio::test]
async fn authorized_read_uses_distinct_proof_and_does_not_claim_final_absence() {
    let repository = AcceptanceTestRepository::default();
    let request = read_request("request", "never-seen");
    let verifier = verifier(
        &request.authorization.decision_id,
        CLOUD_COMPUTE_K8S_OPERATION_GET_SURFACE,
    );
    let lookup = get_cloud_compute_k8s_operation_from_api_with_authorization_verifier(
        &repository,
        request,
        &verifier,
    )
    .await
    .unwrap();

    assert_eq!(lookup, CloudComputeK8sOperationLookup::NotObserved);
    assert_eq!(repository.calls(), (0, 1));
}
