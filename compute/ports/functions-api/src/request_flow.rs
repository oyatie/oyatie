pub fn validate_cloud_compute_functions_invoke_request(
    request: &CloudComputeFunctionsInvokeApiRequest,
) -> Result<ResourceId, CloudComputeFunctionsApiError> {
    validate_cloud_compute_functions_invoke_request_with_optional_authorization_verifier(
        request, None,
    )
}

pub fn validate_cloud_compute_functions_invoke_request_with_authorization_verifier(
    request: &CloudComputeFunctionsInvokeApiRequest,
    authorization_verifier: &CloudComputeFunctionsAuthorizationVerifier,
) -> Result<ResourceId, CloudComputeFunctionsApiError> {
    validate_cloud_compute_functions_invoke_request_with_optional_authorization_verifier(
        request,
        Some(authorization_verifier),
    )
}

fn validate_cloud_compute_functions_invoke_request_with_optional_authorization_verifier(
    request: &CloudComputeFunctionsInvokeApiRequest,
    authorization_verifier: Option<&CloudComputeFunctionsAuthorizationVerifier>,
) -> Result<ResourceId, CloudComputeFunctionsApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_function_id(&request.path_function_id, &request.body.function_id)?;
    let resource_id = validate_function_resource_id(&request.path_function_id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &resource_id,
        &request.body.tenant_id,
    )?;
    validate_authorization(
        authorization_verifier,
        &request.principal,
        &request.authorization.decision_id,
        CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE,
    )?;
    Ok(resource_id)
}

pub fn invoke_cloud_compute_function_from_api(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeFunctionsInvokeIdempotencyLedger,
    request: CloudComputeFunctionsInvokeApiRequest,
) -> Result<CloudComputeFunctionsInvokeSuccessResponse, CloudComputeFunctionsApiError> {
    validate_cloud_compute_functions_invoke_request(&request)?;
    invoke_validated_cloud_compute_function_from_api(catalog, idempotency_ledger, request)
}

pub fn invoke_cloud_compute_function_from_api_with_authorization_verifier(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeFunctionsInvokeIdempotencyLedger,
    authorization_verifier: &CloudComputeFunctionsAuthorizationVerifier,
    request: CloudComputeFunctionsInvokeApiRequest,
) -> Result<CloudComputeFunctionsInvokeSuccessResponse, CloudComputeFunctionsApiError> {
    validate_cloud_compute_functions_invoke_request_with_authorization_verifier(
        &request,
        authorization_verifier,
    )?;
    invoke_validated_cloud_compute_function_from_api(catalog, idempotency_ledger, request)
}

fn invoke_validated_cloud_compute_function_from_api(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeFunctionsInvokeIdempotencyLedger,
    request: CloudComputeFunctionsInvokeApiRequest,
) -> Result<CloudComputeFunctionsInvokeSuccessResponse, CloudComputeFunctionsApiError> {
    let input = function_invoke_input(&request.boundary, &request.body)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE,
    );
    let fingerprint = function_invoke_fingerprint_for(&request.path_function_id, &input);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudComputeFunctionsApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = catalog
        .invoke_function(input)
        .map_err(CloudComputeFunctionsApiError::Compute)
        .map(|receipt| {
            CloudComputeFunctionsInvokeSuccessResponse::accepted(
                invocation_receipt(receipt),
                request_id,
            )
        });
    idempotency_ledger.remember(
        key,
        CloudComputeFunctionsInvokeLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

/// Stable legacy entrypoint for `cloud.compute.functions.invoke`.
///
/// This entrypoint intentionally fails closed because it has no compute-owned
/// authorization verifier. Use `invoke_with_authorization_verifier` for live
/// API-boundary invocation.
pub fn invoke(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeFunctionsInvokeIdempotencyLedger,
    request: CloudComputeFunctionsInvokeApiRequest,
) -> Result<CloudComputeFunctionsInvokeSuccessResponse, CloudComputeFunctionsApiError> {
    invoke_cloud_compute_function_from_api(catalog, idempotency_ledger, request)
}

pub fn invoke_with_authorization_verifier(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeFunctionsInvokeIdempotencyLedger,
    authorization_verifier: &CloudComputeFunctionsAuthorizationVerifier,
    request: CloudComputeFunctionsInvokeApiRequest,
) -> Result<CloudComputeFunctionsInvokeSuccessResponse, CloudComputeFunctionsApiError> {
    invoke_cloud_compute_function_from_api_with_authorization_verifier(
        catalog,
        idempotency_ledger,
        authorization_verifier,
        request,
    )
}

fn validate_boundary(
    boundary: &CloudComputeFunctionsApiBoundaryContext,
) -> Result<(), CloudComputeFunctionsApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudComputeFunctionsApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudComputeFunctionsApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudComputeFunctionsApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_function_id(
    path_function_id: &str,
    body_function_id: &str,
) -> Result<(), CloudComputeFunctionsApiError> {
    if path_function_id.trim().is_empty() {
        return Err(CloudComputeFunctionsApiError::EmptyPathFunctionId);
    }
    if path_function_id != body_function_id {
        return Err(CloudComputeFunctionsApiError::FunctionIdMismatch {
            path_function_id: path_function_id.to_string(),
            body_function_id: body_function_id.to_string(),
        });
    }
    Ok(())
}

fn validate_function_resource_id(value: &str) -> Result<ResourceId, CloudComputeFunctionsApiError> {
    let id = ResourceId::new(value.to_string()).map_err(|_| {
        CloudComputeFunctionsApiError::InvalidFunctionId {
            function_id: value.to_string(),
        }
    })?;
    let kind_label =
        id.kind_label()
            .map_err(|_| CloudComputeFunctionsApiError::InvalidFunctionId {
                function_id: value.to_string(),
            })?;
    if kind_label != "function" {
        return Err(CloudComputeFunctionsApiError::FunctionKindMismatch {
            function_id: value.to_string(),
            kind_label,
        });
    }
    Ok(id)
}

fn validate_tenant_binding(
    boundary: &CloudComputeFunctionsApiBoundaryContext,
    principal: &CloudComputeFunctionsApiPrincipal,
    resource_id: &ResourceId,
    body_tenant_id: &str,
) -> Result<(), CloudComputeFunctionsApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudComputeFunctionsApiError::EmptyPrincipalId);
    }
    let resource_tenant_id =
        resource_id
            .tenant_id()
            .map_err(|_| CloudComputeFunctionsApiError::InvalidFunctionId {
                function_id: resource_id.value.clone(),
            })?;
    if boundary.tenant_id != principal.tenant_id
        || boundary.tenant_id != resource_tenant_id
        || boundary.tenant_id != body_tenant_id
    {
        return Err(CloudComputeFunctionsApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            resource_tenant_id,
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_authorization(
    authorization_verifier: Option<&CloudComputeFunctionsAuthorizationVerifier>,
    principal: &CloudComputeFunctionsApiPrincipal,
    decision_id: &str,
    surface: &str,
) -> Result<(), CloudComputeFunctionsApiError> {
    if decision_id.trim().is_empty() {
        return Err(CloudComputeFunctionsApiError::EmptyAuthorizationDecisionId);
    }
    let verifier = authorization_verifier
        .ok_or(CloudComputeFunctionsApiError::AuthorizationVerifierMissing)?;
    verifier.verify(principal, decision_id, surface)
}

fn function_invoke_input(
    boundary: &CloudComputeFunctionsApiBoundaryContext,
    body: &CloudComputeFunctionsInvokeRequest,
) -> Result<FunctionInvocationRequest, CloudComputeFunctionsApiError> {
    Ok(FunctionInvocationRequest {
        invocation_id: body.invocation_id.clone(),
        tenant_id: body.tenant_id.clone(),
        function_id: body.function_id.clone(),
        region: body.region.clone(),
        payload_data_class: parse_api_data_class(body.payload_data_class.clone())?,
        idempotency_key: boundary.idempotency_key.clone(),
        current_concurrent_invocations: body.current_concurrent_invocations,
        requested_at_epoch_seconds: body.requested_at_epoch_seconds,
    })
}
