const TENANT_ID_PREFIX: &str = "ten_";
const MAX_OBJECT_KEY_LEN: usize = 1024;

fn validate_provider_ref(
    value: &str,
    error: StorageProviderObjectError,
) -> Result<(), StorageProviderObjectError> {
    if value.trim().is_empty()
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_provider_block_ref(
    value: &str,
    error: StorageProviderBlockError,
) -> Result<(), StorageProviderBlockError> {
    if value.trim().is_empty()
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_tenant_id(value: &str) -> Result<(), CloudStorageError> {
    let Some(suffix) = value.strip_prefix(TENANT_ID_PREFIX) else {
        return Err(CloudStorageError::InvalidTenantId);
    };
    if suffix.is_empty()
        || suffix.starts_with('-')
        || suffix.ends_with('-')
        || suffix.contains("--")
        || !suffix.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
    {
        return Err(CloudStorageError::InvalidTenantId);
    }
    Ok(())
}

fn validate_bucket_resource(value: &str, tenant_id: &str) -> Result<(), CloudStorageError> {
    let bucket_id = ResourceId::new(value.to_string()).map_err(map_resource_error)?;
    if bucket_id.tenant_id().map_err(map_resource_error)? != tenant_id {
        return Err(CloudStorageError::ResourceTenantMismatch);
    }
    if bucket_id.kind_label().map_err(map_resource_error)? != "bucket" {
        return Err(CloudStorageError::ResourceKindMismatch);
    }
    Ok(())
}

fn validate_object_key(value: &str) -> Result<(), CloudStorageError> {
    if value.is_empty()
        || value.len() > MAX_OBJECT_KEY_LEN
        || value.starts_with('/')
        || value.split('/').any(|segment| segment == "..")
        || value.bytes().any(|byte| byte.is_ascii_control())
    {
        Err(CloudStorageError::InvalidObjectKey)
    } else {
        Ok(())
    }
}

fn validate_etag(value: &str) -> Result<(), CloudStorageError> {
    let unquoted = value.trim_matches('"');
    if unquoted.len() == 32 && unquoted.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(CloudStorageError::InvalidEtag)
    }
}

fn privacy_class(data_class: DataClass) -> Result<PrivacyDataClass, CloudStorageError> {
    PrivacyDataClass::new(data_class).map_err(|_| CloudStorageError::InvalidDataClass)
}

fn validate_size(value: u64) -> Result<(), CloudStorageError> {
    if value > 0 {
        Ok(())
    } else {
        Err(CloudStorageError::InvalidSize)
    }
}

fn validate_performance(value: VolumePerformance) -> Result<(), CloudStorageError> {
    if value.iops > 0 && value.throughput_mbps > 0 {
        Ok(())
    } else {
        Err(CloudStorageError::InvalidPerformance)
    }
}

fn validate_canonical_segment(
    value: &str,
    error: CloudStorageError,
) -> Result<(), CloudStorageError> {
    if value.trim().is_empty()
        || value.starts_with('-')
        || value.ends_with('-')
        || value.contains("--")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_az_region(az: &AzCode, region: &RegionCode) -> Result<(), CloudStorageError> {
    if az.value == region.value
        || az
            .value
            .strip_prefix(&region.value)
            .is_some_and(|suffix| suffix.starts_with('-') && suffix.len() > 1)
    {
        Ok(())
    } else {
        Err(CloudStorageError::AzRegionMismatch)
    }
}

fn validate_cell_location(
    cell_id: &CellId,
    region: &RegionCode,
    az: Option<&AzCode>,
) -> Result<(), CloudStorageError> {
    let expected_prefix = match az {
        Some(az) => format!("cell-{}-", az.value),
        None => format!("cell-{}-", region.value),
    };
    if cell_id.value.starts_with(&expected_prefix) {
        Ok(())
    } else {
        Err(CloudStorageError::CellLocationMismatch)
    }
}

fn validate_residency_allows_region(
    residency: &ResidencyClass,
    region: &RegionCode,
) -> Result<(), CloudStorageError> {
    if residency_class_allows_home_region_label(residency, &region.value) {
        Ok(())
    } else {
        Err(CloudStorageError::ReplicationResidencyDenied)
    }
}

fn resource_id_for(
    value: &str,
    tenant_id: &str,
    region: &RegionCode,
    kind: ResourceKind,
) -> Result<ResourceId, CloudStorageError> {
    let id = ResourceId::new(value.to_string()).map_err(map_resource_error)?;
    if id.tenant_id().map_err(map_resource_error)? != tenant_id {
        return Err(CloudStorageError::ResourceTenantMismatch);
    }
    if id.region().map_err(map_resource_error)? != *region {
        return Err(CloudStorageError::ResourceRegionMismatch);
    }
    if id.kind_label().map_err(map_resource_error)? != kind.type_label() {
        return Err(CloudStorageError::ResourceKindMismatch);
    }
    Ok(id)
}

fn validate_encryption_key(
    mode: EncryptionMode,
    key: Option<&str>,
    region: &RegionCode,
    tenant_id: &str,
) -> Result<(), CloudStorageError> {
    let expected_origin = match mode {
        EncryptionMode::Sse => None,
        EncryptionMode::SseKms => Some(KmsKeyOrigin::OyatieManaged),
        EncryptionMode::Byok => Some(KmsKeyOrigin::Byok),
        EncryptionMode::Hyok => Some(KmsKeyOrigin::Hyok),
    };
    let Some(expected_origin) = expected_origin else {
        return if key.is_none() {
            Ok(())
        } else {
            Err(CloudStorageError::UnexpectedKmsKey)
        };
    };
    let Some(key) = key else {
        return Err(CloudStorageError::MissingKmsKey);
    };
    let key = KmsKeyId::new(key.to_string()).map_err(|_| CloudStorageError::InvalidKmsKeyId)?;
    if key
        .origin()
        .map_err(|_| CloudStorageError::InvalidKmsKeyId)?
        != expected_origin
    {
        return Err(CloudStorageError::KmsKeyModeMismatch);
    }
    if key
        .region()
        .map_err(|_| CloudStorageError::InvalidKmsKeyId)?
        != *region
    {
        return Err(CloudStorageError::KmsKeyRegionMismatch);
    }
    if key
        .tenant_id()
        .map_err(|_| CloudStorageError::InvalidKmsKeyId)?
        != tenant_id
    {
        return Err(CloudStorageError::KmsKeyTenantMismatch);
    }
    Ok(())
}

fn map_resource_error(error: CloudResourceError) -> CloudStorageError {
    match error {
        CloudResourceError::InvalidResourceId => CloudStorageError::InvalidResourceId,
        CloudResourceError::ResourceIdTenantMismatch => CloudStorageError::ResourceTenantMismatch,
        CloudResourceError::ResourceIdRegionMismatch => CloudStorageError::ResourceRegionMismatch,
        CloudResourceError::ResourceIdKindMismatch => CloudStorageError::ResourceKindMismatch,
        CloudResourceError::InvalidTenantId => CloudStorageError::InvalidTenantId,
        _ => CloudStorageError::InvalidResourceId,
    }
}
