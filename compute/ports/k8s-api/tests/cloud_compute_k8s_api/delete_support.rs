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
        allowed_surfaces: surfaces.iter().map(|s| (*s).to_string()).collect(),
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

fn delete_cloud_compute_k8s_cluster_from_api(
    catalog: &CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sDeleteIdempotencyLedger,
    request: CloudComputeK8sClusterDeleteApiRequest,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    let verifier = trusted_delete_verifier_for(&request);
    delete_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        catalog,
        idempotency_ledger,
        request,
        &verifier,
    )
}

fn delete_cluster(
    catalog: &CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sDeleteIdempotencyLedger,
    request: CloudComputeK8sClusterDeleteApiRequest,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    let verifier = trusted_delete_verifier_for(&request);
    delete_cluster_with_authorization_verifier(catalog, idempotency_ledger, request, &verifier)
}

/// Populate the catalog with one cluster so delete tests have something to find.
fn catalog_with_cluster() -> (CloudComputeCatalog, CloudComputeK8sCreateIdempotencyLedger) {
    let mut catalog = CloudComputeCatalog::default();
    let mut create_ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    create_cloud_compute_k8s_cluster_from_api(
        &mut catalog,
        &mut create_ledger,
        request("req-setup-delete", "idem-setup-delete"),
    )
    .expect("setup cluster create succeeds");
    (catalog, create_ledger)
}
