/// Validates all boundary conditions for a delete request without touching the
/// catalog.
///
/// Returns the parsed [`ResourceId`] on success so the caller can use it for
/// catalog lookup.
pub fn validate_cloud_compute_k8s_cluster_delete_request(
    request: &CloudComputeK8sClusterDeleteApiRequest,
) -> Result<ResourceId, CloudComputeK8sApiError> {
    validate_cloud_compute_k8s_cluster_delete_request_with_authorization_verifier(
        request,
        &CloudComputeK8sFailClosedAuthorizationVerifier,
    )
}

pub fn validate_cloud_compute_k8s_cluster_delete_request_with_authorization_verifier(
    request: &CloudComputeK8sClusterDeleteApiRequest,
    authorization_verifier: &impl CloudComputeK8sAuthorizationVerifier,
) -> Result<ResourceId, CloudComputeK8sApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_cluster_id_only(&request.path_cluster_id)?;
    let resource_id = validate_cluster_resource_id(&request.path_cluster_id)?;
    validate_delete_tenant_binding(&request.boundary, &request.principal, &resource_id)?;
    validate_authorization(
        &request.principal,
        &request.authorization.decision_id,
        CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
        authorization_verifier,
    )?;
    Ok(resource_id)
}

/// Full delete execution: validates, checks idempotency, looks up the cluster,
/// projects its state to `Deleting`, records the ledger entry, and returns the
/// typed success response.
///
/// The catalog is accessed read-only — actual teardown is the reconciler's
/// concern. Only the boundary-owned idempotency ledger is mutated.
pub fn delete_cloud_compute_k8s_cluster_from_api(
    catalog: &CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sDeleteIdempotencyLedger,
    request: CloudComputeK8sClusterDeleteApiRequest,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    let resource_id = validate_cloud_compute_k8s_cluster_delete_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
    );
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.path_cluster_id == request.path_cluster_id {
            return entry.result.clone();
        }
        return Err(CloudComputeK8sApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let cluster = catalog
        .kubernetes_clusters()
        .find(|c| c.resource_id.value == resource_id)
        .ok_or_else(|| CloudComputeK8sApiError::ClusterNotFound {
            cluster_id: request.path_cluster_id.clone(),
        })?;

    let mut record = cluster_record(cluster.clone());
    record.state = cluster_state_label(KubernetesClusterState::Draining).to_string();

    let request_id = request.boundary.request_id.clone();
    let result: CloudComputeK8sDeleteApiResult = Ok(
        CloudComputeK8sClusterDeleteSuccessResponse::accepted(record, request_id),
    );

    idempotency_ledger.remember(
        key,
        CloudComputeK8sDeleteLedgerEntry {
            path_cluster_id: request.path_cluster_id,
            result: result.clone(),
        },
    );
    result
}

pub fn delete_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
    catalog: &CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sDeleteIdempotencyLedger,
    request: CloudComputeK8sClusterDeleteApiRequest,
    authorization_verifier: &impl CloudComputeK8sAuthorizationVerifier,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    let resource_id =
        validate_cloud_compute_k8s_cluster_delete_request_with_authorization_verifier(
            &request,
            authorization_verifier,
        )?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
    );
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.path_cluster_id == request.path_cluster_id {
            return entry.result.clone();
        }
        return Err(CloudComputeK8sApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let cluster = catalog
        .kubernetes_clusters()
        .find(|c| c.resource_id.value == resource_id)
        .ok_or_else(|| CloudComputeK8sApiError::ClusterNotFound {
            cluster_id: request.path_cluster_id.clone(),
        })?;

    let mut record = cluster_record(cluster.clone());
    record.state = cluster_state_label(KubernetesClusterState::Draining).to_string();

    let request_id = request.boundary.request_id.clone();
    let result: CloudComputeK8sDeleteApiResult = Ok(
        CloudComputeK8sClusterDeleteSuccessResponse::accepted(record, request_id),
    );

    idempotency_ledger.remember(
        key,
        CloudComputeK8sDeleteLedgerEntry {
            path_cluster_id: request.path_cluster_id,
            result: result.clone(),
        },
    );
    result
}

/// Stable planned entrypoint for `cloud.compute.k8s.cluster.delete`.
///
/// Delegates to [`delete_cloud_compute_k8s_cluster_from_api`] so the plan
/// symbol remains stable without adding a second validation path.
pub fn delete_cluster(
    catalog: &CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sDeleteIdempotencyLedger,
    request: CloudComputeK8sClusterDeleteApiRequest,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    delete_cloud_compute_k8s_cluster_from_api(catalog, idempotency_ledger, request)
}

pub fn delete_cluster_with_authorization_verifier(
    catalog: &CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sDeleteIdempotencyLedger,
    request: CloudComputeK8sClusterDeleteApiRequest,
    authorization_verifier: &impl CloudComputeK8sAuthorizationVerifier,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    delete_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        catalog,
        idempotency_ledger,
        request,
        authorization_verifier,
    )
}

/// Validates that `path_cluster_id` is non-empty (delete has no body to match
/// against).
fn validate_path_cluster_id_only(path_cluster_id: &str) -> Result<(), CloudComputeK8sApiError> {
    if path_cluster_id.trim().is_empty() {
        return Err(CloudComputeK8sApiError::EmptyPathClusterId);
    }
    Ok(())
}

/// Validates that the tenant encoded in the cluster resource-id matches both
/// the boundary tenant header and the authenticated principal tenant.
///
/// Returns `EmptyPrincipalId` (401) if the principal id is absent, and
/// `TenantMismatch` (403) if any tenant comparison fails.
fn validate_delete_tenant_binding(
    boundary: &CloudComputeK8sApiBoundaryContext,
    principal: &CloudComputeK8sApiPrincipal,
    resource_id: &ResourceId,
) -> Result<(), CloudComputeK8sApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudComputeK8sApiError::EmptyPrincipalId);
    }
    let resource_tenant_id =
        resource_id
            .tenant_id()
            .map_err(|_| CloudComputeK8sApiError::InvalidClusterId {
                cluster_id: resource_id.value.clone(),
            })?;
    if boundary.tenant_id != principal.tenant_id || boundary.tenant_id != resource_tenant_id {
        return Err(CloudComputeK8sApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            resource_tenant_id,
            body_tenant_id: String::new(),
        });
    }
    Ok(())
}
