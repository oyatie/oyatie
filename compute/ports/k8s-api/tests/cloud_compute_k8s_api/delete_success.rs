#[test]
fn k8s_delete_api_surface_constants_and_status_codes() {
    assert_eq!(
        CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
        "cloud.compute.k8s.cluster.delete"
    );
    assert_eq!(CloudComputeK8sClusterDeleteApiStatus::Accepted.code(), 202);
    assert_eq!(
        CloudComputeK8sClusterDeleteApiStatus::BadRequest.code(),
        400
    );
    assert_eq!(
        CloudComputeK8sClusterDeleteApiStatus::Unauthorized.code(),
        401
    );
    assert_eq!(CloudComputeK8sClusterDeleteApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudComputeK8sClusterDeleteApiStatus::NotFound.code(), 404);
    assert_eq!(
        CloudComputeK8sClusterDeleteApiStatus::UnprocessableEntity.code(),
        422
    );
    assert_eq!(
        CloudComputeK8sClusterDeleteApiStatus::ServiceUnavailable.code(),
        503
    );
}

#[test]
fn k8s_delete_api_accepts_intent_without_forging_observed_state() {
    let mut repository = delete_repository_with_cluster();

    let response = delete_cloud_compute_k8s_cluster_from_api(
        &mut repository,
        delete_request("req-del-happy", "idem-del-happy"),
    )
    .expect("authorized cluster delete succeeds");

    assert_eq!(response.metadata.request_id, "req-del-happy");
    assert_eq!(response.data.resource_id, CLUSTER_ID);
    assert_eq!(response.data.tenant_id, "ten_alpha");
    assert_eq!(response.data.state, "creating");
    assert_eq!(response.data.desired_state, "deleted");
    assert_eq!(response.data.schema_version, 2);
    assert_eq!(
        stored_cluster_lifecycle(&repository),
        (
            KubernetesClusterState::Creating,
            KubernetesClusterDesiredState::Deleted,
        )
    );
    assert_eq!(repository.entry_count(), 1);
}

#[test]
fn k8s_delete_api_replay_returns_same_response_without_double_teardown() {
    let mut repository = delete_repository_with_cluster();

    let first = delete_cloud_compute_k8s_cluster_from_api(
        &mut repository,
        delete_request("req-del-idem-1", "idem-del-replay"),
    )
    .expect("first delete accepted");

    let mut retry = delete_request("req-del-idem-2", "idem-del-replay");
    retry.authorization.decision_id = "authz_del_sp_compute_refreshed".to_string();
    retry.authorization.proof = Some(authorization_proof_for(
        "sp_compute",
        CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
        &retry.authorization.decision_id,
    ));
    let second = delete_cloud_compute_k8s_cluster_from_api(&mut repository, retry)
        .expect("same idempotency key replays");

    assert_eq!(first, second);
    assert_eq!(repository.entry_count(), 1);
}

#[test]
fn k8s_delete_api_stable_entrypoint_delegates() {
    let mut repository = delete_repository_with_cluster();

    let response = delete_cluster(
        &mut repository,
        delete_request("req-del-alias", "idem-del-alias"),
    )
    .expect("stable delete_cluster entrypoint succeeds");

    assert_eq!(response.metadata.request_id, "req-del-alias");
    assert_eq!(response.data.state, "creating");
    assert_eq!(response.data.desired_state, "deleted");
    assert_eq!(repository.entry_count(), 1);
}

#[test]
fn k8s_delete_api_rejects_empty_request_id() {
    let mut repository = delete_repository_with_cluster();
    let mut req = delete_request("req-del-empty-rid", "idem-del-empty-rid");
    req.boundary.request_id.clear();

    let error = delete_cloud_compute_k8s_cluster_from_api(&mut repository, req)
        .expect_err("empty request_id rejected");

    assert_eq!(error, CloudComputeK8sApiError::EmptyRequestId);
    assert_eq!(error.cluster_delete_status_code(), 400);
    assert!(repository.is_empty());
}

#[test]
fn k8s_delete_api_rejects_empty_tenant() {
    let mut repository = delete_repository_with_cluster();
    let mut req = delete_request("req-del-empty-ten", "idem-del-empty-ten");
    req.boundary.tenant_id.clear();

    let error = delete_cloud_compute_k8s_cluster_from_api(&mut repository, req)
        .expect_err("empty tenant rejected");

    assert_eq!(error, CloudComputeK8sApiError::EmptyTenantHeader);
    assert_eq!(error.cluster_delete_status_code(), 400);
    assert!(repository.is_empty());
}

#[test]
fn k8s_delete_api_rejects_empty_idempotency_key() {
    let mut repository = delete_repository_with_cluster();
    let mut req = delete_request("req-del-empty-idem", "idem-del-empty-idem");
    req.boundary.idempotency_key.clear();

    let error = delete_cloud_compute_k8s_cluster_from_api(&mut repository, req)
        .expect_err("empty idempotency_key rejected");

    assert_eq!(error, CloudComputeK8sApiError::EmptyIdempotencyKey);
    assert_eq!(error.cluster_delete_status_code(), 400);
    assert!(repository.is_empty());
}

#[test]
fn k8s_delete_api_rejects_empty_principal_as_unauthorized() {
    let mut repository = delete_repository_with_cluster();
    let mut req = delete_request("req-del-empty-prin", "idem-del-empty-prin");
    req.principal.principal_id.clear();

    let error = delete_cloud_compute_k8s_cluster_from_api(&mut repository, req)
        .expect_err("empty principal_id is 401");

    assert_eq!(error, CloudComputeK8sApiError::EmptyPrincipalId);
    assert_eq!(error.cluster_delete_status_code(), 401);
    assert!(repository.is_empty());
}

#[test]
fn k8s_delete_api_legacy_entrypoint_fails_closed_without_authorization_verifier() {
    let mut repository = delete_repository_with_cluster();
    let req = delete_request("req-del-missing-verifier", "idem-del-missing-verifier");

    let error = delete_cloud_compute_k8s_cluster_from_api_without_authorization_verifier(
        &mut repository,
        req,
    )
    .expect_err("legacy delete entrypoint has no trusted authorization verifier");

    assert_eq!(
        error,
        CloudComputeK8sApiError::AuthorizationDenied {
            surface: CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE.to_string(),
        }
    );
    assert_eq!(error.cluster_delete_status_code(), 403);
    let planned_error = delete_cluster_without_authorization_verifier(
        &mut repository,
        delete_request(
            "req-del-planned-missing-verifier",
            "idem-del-planned-missing-verifier",
        ),
    )
    .expect_err("legacy planned delete entrypoint has no trusted authorization verifier");
    assert_eq!(planned_error, error);
    assert!(repository.is_empty());
}
