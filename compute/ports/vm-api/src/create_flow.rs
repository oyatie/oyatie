pub fn validate_cloud_compute_vm_create_request(
    request: &CloudComputeVmCreateApiRequest,
) -> Result<ResourceId, CloudComputeVmApiError> {
    validate_cloud_compute_vm_create_request_with_verifier(
        request,
        &CloudComputeVmMissingAuthorizationVerifier,
    )
}

pub fn validate_cloud_compute_vm_create_request_with_verifier(
    request: &CloudComputeVmCreateApiRequest,
    authorization_verifier: &impl CloudComputeVmApiAuthorizationVerifier,
) -> Result<ResourceId, CloudComputeVmApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_instance_id(&request.path_instance_id, &request.body.resource_id)?;
    let resource_id = validate_instance_resource_id(&request.path_instance_id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &resource_id,
        &request.body.tenant_id,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization.decision_id,
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
        authorization_verifier,
    )?;
    Ok(resource_id)
}

pub fn create_cloud_compute_vm_from_api(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeVmCreateIdempotencyLedger,
    request: CloudComputeVmCreateApiRequest,
) -> Result<CloudComputeVmCreateSuccessResponse, CloudComputeVmApiError> {
    create_cloud_compute_vm_from_api_with_verifier(
        catalog,
        idempotency_ledger,
        request,
        &CloudComputeVmMissingAuthorizationVerifier,
    )
}

pub fn create_cloud_compute_vm_from_api_with_verifier(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeVmCreateIdempotencyLedger,
    request: CloudComputeVmCreateApiRequest,
    authorization_verifier: &impl CloudComputeVmApiAuthorizationVerifier,
) -> Result<CloudComputeVmCreateSuccessResponse, CloudComputeVmApiError> {
    validate_cloud_compute_vm_create_request_with_verifier(&request, authorization_verifier)?;
    let input = instance_create_input(&request.body)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
    );
    let fingerprint = vm_create_fingerprint_for(&request.path_instance_id, &input);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudComputeVmApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = catalog
        .create_instance(input)
        .map_err(CloudComputeVmApiError::Compute)
        .map(|instance| {
            CloudComputeVmCreateSuccessResponse::created(vm_record(instance), request_id)
        });
    idempotency_ledger.remember(
        key,
        CloudComputeVmCreateLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_boundary(
    boundary: &CloudComputeVmApiBoundaryContext,
) -> Result<(), CloudComputeVmApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudComputeVmApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudComputeVmApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudComputeVmApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_instance_id(
    path_instance_id: &str,
    body_resource_id: &str,
) -> Result<(), CloudComputeVmApiError> {
    if path_instance_id.trim().is_empty() {
        return Err(CloudComputeVmApiError::EmptyPathInstanceId);
    }
    if path_instance_id != body_resource_id {
        return Err(CloudComputeVmApiError::InstanceIdMismatch {
            path_instance_id: path_instance_id.to_string(),
            body_resource_id: body_resource_id.to_string(),
        });
    }
    Ok(())
}

fn validate_instance_resource_id(value: &str) -> Result<ResourceId, CloudComputeVmApiError> {
    let id = ResourceId::new(value.to_string()).map_err(|_| {
        CloudComputeVmApiError::InvalidInstanceId {
            instance_id: value.to_string(),
        }
    })?;
    let kind_label = id
        .kind_label()
        .map_err(|_| CloudComputeVmApiError::InvalidInstanceId {
            instance_id: value.to_string(),
        })?;
    if kind_label != "instance" {
        return Err(CloudComputeVmApiError::InstanceKindMismatch {
            instance_id: value.to_string(),
            kind_label,
        });
    }
    Ok(id)
}

fn validate_tenant_binding(
    boundary: &CloudComputeVmApiBoundaryContext,
    principal: &CloudComputeVmApiPrincipal,
    resource_id: &ResourceId,
    body_tenant_id: &str,
) -> Result<(), CloudComputeVmApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudComputeVmApiError::EmptyPrincipalId);
    }
    let resource_tenant_id =
        resource_id
            .tenant_id()
            .map_err(|_| CloudComputeVmApiError::InvalidInstanceId {
                instance_id: resource_id.value.clone(),
            })?;
    if boundary.tenant_id != principal.tenant_id
        || boundary.tenant_id != resource_tenant_id
        || boundary.tenant_id != body_tenant_id
    {
        return Err(CloudComputeVmApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            resource_tenant_id,
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudComputeVmApiPrincipal,
    decision_id: &str,
    surface: &str,
    authorization_verifier: &impl CloudComputeVmApiAuthorizationVerifier,
) -> Result<(), CloudComputeVmApiError> {
    if decision_id.trim().is_empty() {
        return Err(CloudComputeVmApiError::EmptyAuthorizationDecisionId);
    }
    validate_authorization_proof(principal, decision_id, surface, authorization_verifier)
}

fn validate_authorization_proof(
    principal: &CloudComputeVmApiPrincipal,
    decision_id: &str,
    surface: &str,
    authorization_verifier: &impl CloudComputeVmApiAuthorizationVerifier,
) -> Result<(), CloudComputeVmApiError> {
    let Some(proof) = authorization_verifier.proof_for_decision(decision_id) else {
        return Err(CloudComputeVmApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    };
    let evaluation_epoch_seconds = authorization_verifier.evaluation_epoch_seconds();
    if !proof.verified
        || proof.tenant_id != principal.tenant_id
        || proof.principal_id != principal.principal_id
        || proof.surface != surface
        || proof.decision_id != decision_id
        || proof.issued_at_epoch_seconds >= proof.expires_at_epoch_seconds
        || evaluation_epoch_seconds < proof.issued_at_epoch_seconds
        || evaluation_epoch_seconds >= proof.expires_at_epoch_seconds
    {
        return Err(CloudComputeVmApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}
