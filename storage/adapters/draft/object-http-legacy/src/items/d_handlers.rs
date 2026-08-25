pub fn validate_cloud_storage_object_put_request(
    request: &CloudStorageObjectPutApiRequest,
) -> Result<(), CloudStorageObjectApiError> {
    validate_mutation_boundary(&request.boundary)?;
    validate_path_body_binding(
        &request.path_bucket_id,
        &request.path_object_key,
        &request.body.bucket_id,
        &request.body.key,
    )?;
    let bucket_id = validate_bucket_id(&request.path_bucket_id)?;
    validate_object_key(&request.path_object_key)?;
    validate_tenant_binding(
        &request.boundary.tenant_id,
        &request.principal,
        &bucket_id,
        Some(&request.body.tenant_id),
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_STORAGE_OBJECT_PUT_SURFACE,
    )
}

pub fn put_cloud_storage_object_from_api(
    catalog: &mut CloudStorageCatalog,
    idempotency_ledger: &mut CloudStorageObjectPutIdempotencyLedger,
    request: CloudStorageObjectPutApiRequest,
) -> Result<CloudStorageObjectPutSuccessResponse, CloudStorageObjectApiError> {
    validate_cloud_storage_object_put_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_STORAGE_OBJECT_PUT_SURFACE,
    );
    let fingerprint = put_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        return match replay_outcome_for(entry, &fingerprint, &request.boundary.idempotency_key) {
            CloudStorageObjectReplayOutcome::Replayed { response } => Ok(*response),
            CloudStorageObjectReplayOutcome::Conflict { idempotency_key } => {
                Err(CloudStorageObjectApiError::IdempotencyKeyReused { idempotency_key })
            }
        };
    }

    let request_id = request.boundary.request_id.clone();
    let result = object_put_input(request.body)
        .and_then(|input| {
            catalog
                .put_object(input)
                .map_err(CloudStorageObjectApiError::Storage)
        })
        .map(|object| {
            CloudStorageObjectPutSuccessResponse::created(object_record(object), request_id)
        });
    idempotency_ledger.entries.insert(
        key,
        CloudStorageObjectPutLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

pub fn validate_cloud_storage_object_get_request(
    request: &CloudStorageObjectGetApiRequest,
) -> Result<ResourceId, CloudStorageObjectApiError> {
    validate_read_boundary(&request.boundary)?;
    let bucket_id = validate_bucket_id(&request.path_bucket_id)?;
    validate_object_key(&request.path_object_key)?;
    validate_tenant_binding(
        &request.boundary.tenant_id,
        &request.principal,
        &bucket_id,
        None,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_STORAGE_OBJECT_GET_SURFACE,
    )?;
    Ok(bucket_id)
}

pub fn get_cloud_storage_object_from_api(
    catalog: &CloudStorageCatalog,
    request: CloudStorageObjectGetApiRequest,
) -> Result<CloudStorageObjectGetSuccessResponse, CloudStorageObjectApiError> {
    let bucket_id = validate_cloud_storage_object_get_request(&request)?;
    let request_id = request.boundary.request_id.clone();
    let object = catalog
        .objects()
        .find(|object| {
            object.bucket_id.value == bucket_id && object.key.value.value == request.path_object_key
        })
        .ok_or_else(|| CloudStorageObjectApiError::ObjectNotFound {
            bucket_id: request.path_bucket_id.clone(),
            key: request.path_object_key.clone(),
        })?;
    if object.tenant_id.value != request.boundary.tenant_id {
        return Err(CloudStorageObjectApiError::TenantMismatch {
            header_tenant_id: request.boundary.tenant_id,
            principal_tenant_id: request.principal.tenant_id,
            resource_tenant_id: object.tenant_id.value.clone(),
            body_tenant_id: None,
        });
    }
    Ok(CloudStorageObjectGetSuccessResponse::ok(
        object_record(object.clone()),
        request_id,
    ))
}

fn validate_mutation_boundary(
    boundary: &CloudStorageObjectMutationBoundaryContext,
) -> Result<(), CloudStorageObjectApiError> {
    validate_request_tenant(&boundary.request_id, &boundary.tenant_id)?;
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudStorageObjectApiError::EmptyIdempotencyKey);
    }
    Ok(())
}
fn validate_read_boundary(
    boundary: &CloudStorageObjectReadBoundaryContext,
) -> Result<(), CloudStorageObjectApiError> {
    validate_request_tenant(&boundary.request_id, &boundary.tenant_id)
}

fn validate_request_tenant(
    request_id: &str,
    tenant_id: &str,
) -> Result<(), CloudStorageObjectApiError> {
    if request_id.trim().is_empty() {
        return Err(CloudStorageObjectApiError::EmptyRequestId);
    }
    if tenant_id.trim().is_empty() {
        return Err(CloudStorageObjectApiError::EmptyTenantHeader);
    }
    Ok(())
}

fn validate_path_body_binding(
    path_bucket_id: &str,
    path_object_key: &str,
    body_bucket_id: &str,
    body_key: &str,
) -> Result<(), CloudStorageObjectApiError> {
    if path_bucket_id != body_bucket_id {
        return Err(CloudStorageObjectApiError::BucketIdMismatch {
            path_bucket_id: path_bucket_id.to_string(),
            body_bucket_id: body_bucket_id.to_string(),
        });
    }
    if path_object_key != body_key {
        return Err(CloudStorageObjectApiError::ObjectKeyMismatch {
            path_object_key: path_object_key.to_string(),
            body_key: body_key.to_string(),
        });
    }
    Ok(())
}

fn validate_bucket_id(value: &str) -> Result<ResourceId, CloudStorageObjectApiError> {
    let bucket_id = ResourceId::new(value.to_string()).map_err(|_| {
        CloudStorageObjectApiError::InvalidBucketId {
            bucket_id: value.to_string(),
        }
    })?;
    let kind_label =
        bucket_id
            .kind_label()
            .map_err(|_| CloudStorageObjectApiError::InvalidBucketId {
                bucket_id: value.to_string(),
            })?;
    if kind_label != "bucket" {
        return Err(CloudStorageObjectApiError::BucketKindMismatch {
            bucket_id: value.to_string(),
            kind_label,
        });
    }
    Ok(bucket_id)
}

fn validate_object_key(value: &str) -> Result<ObjectKey, CloudStorageObjectApiError> {
    ObjectKey::new(value.to_string()).map_err(|_| CloudStorageObjectApiError::InvalidObjectKey {
        object_key: value.to_string(),
    })
}

fn validate_tenant_binding(
    header_tenant_id: &str,
    principal: &CloudStorageObjectApiPrincipal,
    bucket_id: &ResourceId,
    body_tenant_id: Option<&str>,
) -> Result<(), CloudStorageObjectApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudStorageObjectApiError::EmptyPrincipalId);
    }
    let resource_tenant_id =
        bucket_id
            .tenant_id()
            .map_err(|_| CloudStorageObjectApiError::InvalidBucketId {
                bucket_id: bucket_id.value.clone(),
            })?;
    if header_tenant_id != principal.tenant_id || header_tenant_id != resource_tenant_id {
        return Err(CloudStorageObjectApiError::TenantMismatch {
            header_tenant_id: header_tenant_id.to_string(),
            principal_tenant_id: principal.tenant_id.clone(),
            resource_tenant_id,
            body_tenant_id: body_tenant_id.map(str::to_string),
        });
    }
    if body_tenant_id.is_some_and(|tenant_id| header_tenant_id != tenant_id) {
        return Err(CloudStorageObjectApiError::TenantMismatch {
            header_tenant_id: header_tenant_id.to_string(),
            principal_tenant_id: principal.tenant_id.clone(),
            resource_tenant_id,
            body_tenant_id: body_tenant_id.map(str::to_string),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudStorageObjectApiPrincipal,
    authorization: &CloudStorageObjectApiAuthorization,
    surface: &str,
) -> Result<(), CloudStorageObjectApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(CloudStorageObjectApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(CloudStorageObjectApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(CloudStorageObjectApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(CloudStorageObjectApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}
