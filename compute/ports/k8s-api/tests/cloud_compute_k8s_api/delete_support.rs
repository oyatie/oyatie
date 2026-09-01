fn delete_boundary_for(
    request_id: &str,
    idempotency_key: &str,
) -> CloudComputeK8sApiBoundaryContext {
    CloudComputeK8sApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn delete_authorization_for(
    principal_id: &str,
    surfaces: &[&str],
) -> CloudComputeK8sApiAuthorization {
    let decision_id = format!("authz_del_{principal_id}");
    CloudComputeK8sApiAuthorization {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: decision_id.clone(),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
        proof: Some(authorization_proof_for(
            principal_id,
            CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
            &decision_id,
        )),
    }
}

fn delete_request(
    request_id: &str,
    idempotency_key: &str,
) -> CloudComputeK8sClusterDeleteApiRequest {
    CloudComputeK8sClusterDeleteApiRequest {
        path_cluster_id: CLUSTER_ID.to_string(),
        boundary: delete_boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_compute"),
        authorization: delete_authorization_for(
            "sp_compute",
            &[CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE],
        ),
    }
}

fn trusted_delete_verifier_for(
    request: &CloudComputeK8sClusterDeleteApiRequest,
) -> CloudComputeK8sTrustedAuthorizationVerifier {
    CloudComputeK8sTrustedAuthorizationVerifier::new(K8S_AUTHZ_EVALUATION_EPOCH_SECONDS)
        .with_authorization_proof(authorization_proof_for(
            &request.principal.principal_id,
            CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
            &request.authorization.decision_id,
        ))
}

async fn delete_cloud_compute_k8s_cluster_from_api(
    repository: &LifecycleTestRepository,
    request: CloudComputeK8sClusterDeleteApiRequest,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    let verifier = trusted_delete_verifier_for(&request);
    delete_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        repository,
        request,
        &verifier,
    )
    .await
}

async fn delete_cluster(
    repository: &LifecycleTestRepository,
    request: CloudComputeK8sClusterDeleteApiRequest,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    let verifier = trusted_delete_verifier_for(&request);
    delete_cluster_with_authorization_verifier(repository, request, &verifier).await
}

async fn delete_repository_with_cluster() -> LifecycleTestRepository {
    let repository = LifecycleTestRepository::default();
    create_cloud_compute_k8s_cluster_from_api(
        &repository,
        request("req-setup-delete", "idem-setup-delete"),
    )
    .await
    .expect("setup cluster create succeeds");
    repository
}

fn stored_cluster_lifecycle(repository: &LifecycleTestRepository) -> (String, String) {
    let cluster = repository
        .cluster_record(CLUSTER_ID)
        .expect("setup cluster remains in the repository");
    (cluster.state, cluster.desired_state)
}
