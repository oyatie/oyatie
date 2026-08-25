pub fn validate_cloud_storage_block_create_request(
    request: &CloudStorageBlockVolumeCreateApiRequest,
) -> Result<(), CloudStorageBlockApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_volume_id(&request.path_volume_id, &request.body.resource_id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &request.body.tenant_id,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        STORAGE_BLOCK_CREATE_SURFACE,
    )
}

pub fn create_cloud_storage_block_volume_from_api(
    catalog: &mut CloudStorageCatalog,
    idempotency_ledger: &mut CloudStorageBlockCreateIdempotencyLedger,
    request: CloudStorageBlockVolumeCreateApiRequest,
) -> Result<CloudStorageBlockVolumeCreateSuccessResponse, CloudStorageBlockApiError> {
    validate_cloud_storage_block_create_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        STORAGE_BLOCK_CREATE_SURFACE,
    );
    let fingerprint = block_create_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudStorageBlockApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = volume_create_input(request.body)
        .and_then(|input| {
            catalog
                .create_volume(input)
                .map_err(CloudStorageBlockApiError::Storage)
        })
        .map(|volume| {
            CloudStorageBlockVolumeCreateSuccessResponse::created(
                block_volume_record(volume),
                request_id,
            )
        });
    idempotency_ledger.entries.insert(
        key,
        CloudStorageBlockCreateIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_boundary(
    boundary: &CloudStorageBlockApiBoundaryContext,
) -> Result<(), CloudStorageBlockApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudStorageBlockApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudStorageBlockApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudStorageBlockApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_volume_id(
    path_volume_id: &str,
    body_resource_id: &str,
) -> Result<(), CloudStorageBlockApiError> {
    if path_volume_id.trim().is_empty() {
        return Err(CloudStorageBlockApiError::EmptyPathVolumeId);
    }
    if path_volume_id != body_resource_id {
        return Err(CloudStorageBlockApiError::VolumeIdMismatch {
            path_volume_id: path_volume_id.to_string(),
            body_resource_id: body_resource_id.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    boundary: &CloudStorageBlockApiBoundaryContext,
    principal: &CloudStorageBlockApiPrincipal,
    body_tenant_id: &str,
) -> Result<(), CloudStorageBlockApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudStorageBlockApiError::EmptyPrincipalId);
    }
    if boundary.tenant_id != principal.tenant_id || boundary.tenant_id != body_tenant_id {
        return Err(CloudStorageBlockApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudStorageBlockApiPrincipal,
    authorization: &CloudStorageBlockApiAuthorization,
    surface: &str,
) -> Result<(), CloudStorageBlockApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(CloudStorageBlockApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(CloudStorageBlockApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(CloudStorageBlockApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(CloudStorageBlockApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn volume_create_input(
    body: CloudStorageBlockVolumeCreateRequest,
) -> Result<VolumeCreate, CloudStorageBlockApiError> {
    Ok(VolumeCreate {
        resource_id: body.resource_id,
        tenant_id: body.tenant_id,
        name: body.name,
        region: body.region,
        az: body.az,
        cell_id: body.cell_id,
        residency: parse_api_residency(body.residency)?,
        tier: parse_api_volume_tier(body.tier)?,
        size_gib: body.size_gib,
        performance: VolumePerformance {
            iops: body.performance.iops,
            throughput_mbps: body.performance.throughput_mbps,
        },
        encryption: parse_api_encryption(body.encryption)?,
        kms_key: body.kms_key,
        data_class: parse_api_data_class(body.data_class)?,
        state: VolumeState::Creating,
        created_at_epoch_seconds: body.created_at_epoch_seconds,
    })
}

fn parse_api_residency(label: String) -> Result<ResidencyClass, CloudStorageBlockApiError> {
    parse_residency_class_label(&label)
        .ok_or(CloudStorageBlockApiError::InvalidResidencyLabel { residency: label })
}

fn parse_api_volume_tier(label: String) -> Result<VolumeTier, CloudStorageBlockApiError> {
    match label.as_str() {
        "general_purpose_ssd" => Ok(VolumeTier::GeneralPurposeSsd),
        "provisioned_iops_ssd" => Ok(VolumeTier::ProvisionedIopsSsd),
        _ => Err(CloudStorageBlockApiError::InvalidVolumeTierLabel { tier: label }),
    }
}

fn parse_api_encryption(label: String) -> Result<EncryptionMode, CloudStorageBlockApiError> {
    match label.as_str() {
        "sse" => Ok(EncryptionMode::Sse),
        "sse_kms" => Ok(EncryptionMode::SseKms),
        "byok" => Ok(EncryptionMode::Byok),
        "hyok" => Ok(EncryptionMode::Hyok),
        _ => Err(CloudStorageBlockApiError::InvalidEncryptionLabel { encryption: label }),
    }
}

fn parse_api_data_class(label: String) -> Result<DataClass, CloudStorageBlockApiError> {
    parse_data_class_label(&label)
        .ok_or(CloudStorageBlockApiError::InvalidDataClassLabel { data_class: label })
}

fn idempotency_key_for(
    boundary: &CloudStorageBlockApiBoundaryContext,
    principal: &CloudStorageBlockApiPrincipal,
    surface: &str,
) -> CloudStorageBlockIdempotencyLedgerKey {
    CloudStorageBlockIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn block_create_fingerprint_for(
    request: &CloudStorageBlockVolumeCreateApiRequest,
) -> CloudStorageBlockRequestFingerprint {
    CloudStorageBlockRequestFingerprint {
        canonical: [
            format!("path.volume_id={}", request.path_volume_id),
            format!("header.tenant_id={}", request.boundary.tenant_id),
            format!("principal.tenant_id={}", request.principal.tenant_id),
            format!("principal.principal_id={}", request.principal.principal_id),
            format!(
                "authorization.tenant_id={}",
                request.authorization.tenant_id
            ),
            format!(
                "authorization.principal_id={}",
                request.authorization.principal_id
            ),
            format!(
                "authorization.decision_id={}",
                request.authorization.decision_id
            ),
            format!(
                "authorization.allowed_surfaces={}",
                request.authorization.allowed_surfaces.join(",")
            ),
            format!("body.resource_id={}", request.body.resource_id),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.name={}", request.body.name),
            format!("body.region={}", request.body.region),
            format!("body.az={}", request.body.az),
            format!("body.cell_id={}", request.body.cell_id),
            format!("body.residency={}", request.body.residency),
            format!("body.tier={}", request.body.tier),
            format!("body.size_gib={}", request.body.size_gib),
            format!("body.performance.iops={}", request.body.performance.iops),
            format!(
                "body.performance.throughput_mbps={}",
                request.body.performance.throughput_mbps
            ),
            format!("body.encryption={}", request.body.encryption),
            format!("body.kms_key={:?}", request.body.kms_key),
            format!("body.data_class={}", request.body.data_class),
            format!(
                "body.created_at_epoch_seconds={}",
                request.body.created_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}
