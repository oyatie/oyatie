use compute_k8s_api::{
    CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE, CLOUD_COMPUTE_K8S_OPERATION_GET_SURFACE,
    CloudComputeK8sAcceptanceApiError, CloudComputeK8sApiAuthorizationProof,
    accept_cloud_compute_k8s_cluster_from_api_with_authorization_verifier,
    get_cloud_compute_k8s_operation_from_api_with_authorization_verifier,
};

use super::acceptance_test_repository::*;

#[tokio::test]
async fn caller_supplied_claims_are_disregarded_in_favor_of_verifier_proof() {
    let repository = AcceptanceTestRepository::default();
    let mut request = pending_request("request", "key");
    request
        .authorization
        .allowed_surfaces
        .push(CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE.into());
    request.authorization.proof = Some(CloudComputeK8sApiAuthorizationProof {
        tenant_id: "ten_alpha".into(),
        principal_id: "sp_compute".into(),
        surface: CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE.into(),
        decision_id: request.authorization.decision_id.clone(),
        verified: true,
        issued_at_epoch_seconds: 1,
        expires_at_epoch_seconds: u64::MAX,
    });
    let error = accept_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        &repository,
        request,
        &compute_k8s_api::CloudComputeK8sTrustedAuthorizationVerifier::default(),
    )
    .await
    .unwrap_err();
    assert!(matches!(
        error,
        CloudComputeK8sAcceptanceApiError::Boundary(_)
    ));
    assert_eq!(repository.calls(), (0, 0));
}

#[tokio::test]
async fn denied_expired_tenant_principal_and_surface_mismatches_never_reach_storage() {
    for mutation in ["denied", "expired", "tenant", "principal", "surface"] {
        let repository = AcceptanceTestRepository::default();
        let request = pending_request("request", mutation);
        let mut proof = CloudComputeK8sApiAuthorizationProof {
            tenant_id: "ten_alpha".into(),
            principal_id: "sp_compute".into(),
            surface: CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE.into(),
            decision_id: request.authorization.decision_id.clone(),
            verified: true,
            issued_at_epoch_seconds: 1_700_099_000,
            expires_at_epoch_seconds: 1_700_100_000,
        };
        match mutation {
            "denied" => proof.verified = false,
            "expired" => proof.expires_at_epoch_seconds = 1_700_099_500,
            "tenant" => proof.tenant_id = "ten_other".into(),
            "principal" => proof.principal_id = "sp_other".into(),
            "surface" => proof.surface = CLOUD_COMPUTE_K8S_OPERATION_GET_SURFACE.into(),
            _ => unreachable!(),
        }
        let verifier =
            compute_k8s_api::CloudComputeK8sTrustedAuthorizationVerifier::new(1_700_099_500)
                .with_authorization_proof(proof);
        let error = accept_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
            &repository,
            request,
            &verifier,
        )
        .await
        .unwrap_err();
        assert!(
            matches!(error, CloudComputeK8sAcceptanceApiError::Boundary(_)),
            "{mutation}: {error:?}"
        );
        assert_eq!(repository.calls(), (0, 0));
    }
}

#[tokio::test]
async fn create_proof_cannot_authorize_operation_read() {
    let repository = AcceptanceTestRepository::default();
    let request = read_request("request", "key");
    let verifier = verifier(
        &request.authorization.decision_id,
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
    );
    let error = get_cloud_compute_k8s_operation_from_api_with_authorization_verifier(
        &repository,
        request,
        &verifier,
    )
    .await
    .unwrap_err();
    assert!(matches!(
        error,
        CloudComputeK8sAcceptanceApiError::Boundary(_)
    ));
    assert_eq!(repository.calls(), (0, 0));
}
