pub async fn accept_cloud_compute_k8s_cluster_from_api(
    repository: &impl CloudComputeK8sAcceptanceRepository,
    request: CloudComputeK8sCreateAcceptanceApiRequest,
) -> Result<CloudComputeK8sCreateAcceptanceResponse, CloudComputeK8sAcceptanceApiError> {
    accept_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        repository, request, &CloudComputeK8sFailClosedAuthorizationVerifier,
    ).await
}

pub async fn accept_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
    repository: &impl CloudComputeK8sAcceptanceRepository,
    request: CloudComputeK8sCreateAcceptanceApiRequest,
    authorization_verifier: &impl CloudComputeK8sAuthorizationVerifier,
) -> Result<CloudComputeK8sCreateAcceptanceResponse, CloudComputeK8sAcceptanceApiError> {
    validate_boundary(&request.boundary).map_err(CloudComputeK8sAcceptanceApiError::Boundary)?;
    validate_path_cluster_id(&request.path_cluster_id, &request.body.resource_id)
        .map_err(CloudComputeK8sAcceptanceApiError::Boundary)?;
    let resource_id = validate_cluster_resource_id(&request.path_cluster_id)
        .map_err(CloudComputeK8sAcceptanceApiError::Boundary)?;
    validate_tenant_binding(&request.boundary, &request.principal, &resource_id, &request.body.tenant_id)
        .map_err(CloudComputeK8sAcceptanceApiError::Boundary)?;
    validate_authorization(&request.principal, &request.authorization.decision_id,
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE, authorization_verifier)
        .map_err(CloudComputeK8sAcceptanceApiError::Boundary)?;
    validate_cloud_compute_k8s_create_intent(&request.body)
        .map_err(CloudComputeK8sAcceptanceApiError::Boundary)?;
    let operation_key = idempotency_key_for(&request.boundary, &request.principal,
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE);
    let expected_intent = request.body.clone();
    let snapshot = repository.accept_create_intent(CloudComputeK8sAcceptCreateIntentCommand {
        operation_key: operation_key.clone(), intent: request.body,
        request_id: request.boundary.request_id,
    }).await.map_err(acceptance_repository_error)?;
    validate_cloud_compute_k8s_operation_snapshot(&snapshot, &operation_key,
        &request.path_cluster_id, Some(&expected_intent)).map_err(acceptance_repository_error)?;
    Ok(CloudComputeK8sCreateAcceptanceResponse { operation: snapshot })
}

pub async fn get_cloud_compute_k8s_operation_from_api(
    repository: &impl CloudComputeK8sAcceptanceRepository,
    request: CloudComputeK8sOperationReadApiRequest,
) -> Result<CloudComputeK8sOperationLookup, CloudComputeK8sAcceptanceApiError> {
    get_cloud_compute_k8s_operation_from_api_with_authorization_verifier(
        repository, request, &CloudComputeK8sFailClosedAuthorizationVerifier,
    ).await
}

pub async fn get_cloud_compute_k8s_operation_from_api_with_authorization_verifier(
    repository: &impl CloudComputeK8sAcceptanceRepository,
    request: CloudComputeK8sOperationReadApiRequest,
    authorization_verifier: &impl CloudComputeK8sAuthorizationVerifier,
) -> Result<CloudComputeK8sOperationLookup, CloudComputeK8sAcceptanceApiError> {
    validate_boundary(&request.boundary).map_err(CloudComputeK8sAcceptanceApiError::Boundary)?;
    validate_path_cluster_id(&request.path_cluster_id, &request.path_cluster_id)
        .map_err(CloudComputeK8sAcceptanceApiError::Boundary)?;
    let resource_id = validate_cluster_resource_id(&request.path_cluster_id)
        .map_err(CloudComputeK8sAcceptanceApiError::Boundary)?;
    validate_tenant_binding(&request.boundary, &request.principal, &resource_id, &request.boundary.tenant_id)
        .map_err(CloudComputeK8sAcceptanceApiError::Boundary)?;
    validate_authorization(&request.principal, &request.authorization.decision_id,
        CLOUD_COMPUTE_K8S_OPERATION_GET_SURFACE, authorization_verifier)
        .map_err(CloudComputeK8sAcceptanceApiError::Boundary)?;
    let operation_key = idempotency_key_for(&request.boundary, &request.principal,
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE);
    let lookup = repository.get_create_operation(CloudComputeK8sReadCreateOperationQuery {
        operation_key: operation_key.clone(), resource_id: request.path_cluster_id.clone(),
    }).await.map_err(acceptance_repository_error)?;
    if let CloudComputeK8sOperationLookup::Found(snapshot) = &lookup {
        validate_cloud_compute_k8s_operation_snapshot(snapshot, &operation_key,
            &request.path_cluster_id, None).map_err(acceptance_repository_error)?;
    }
    Ok(lookup)
}
