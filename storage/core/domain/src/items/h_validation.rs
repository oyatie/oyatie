fn privacy_class_set(
    data_classes: Vec<DataClass>,
) -> Result<BTreeSet<PrivacyDataClass>, CloudStorageError> {
    if data_classes.is_empty() {
        return Err(CloudStorageError::EmptyAllowedDataClassSet);
    }
    let mut typed = BTreeSet::new();
    for data_class in data_classes {
        let data_class = privacy_class(data_class)?;
        if !typed.insert(data_class) {
            return Err(CloudStorageError::DuplicateDataClass);
        }
    }
    Ok(typed)
}

fn validate_object_lock(policy: Option<ObjectLockPolicy>) -> Result<(), CloudStorageError> {
    if policy.is_some_and(|policy| policy.retain_until_epoch_seconds == 0 && !policy.legal_hold) {
        Err(CloudStorageError::InvalidObjectLockPolicy)
    } else {
        Ok(())
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

fn validate_bucket_resource(value: &str, tenant_id: &str) -> Result<ResourceId, CloudStorageError> {
    let bucket_id = ResourceId::new(value.to_string()).map_err(map_resource_error)?;
    if bucket_id.tenant_id().map_err(map_resource_error)? != tenant_id {
        return Err(CloudStorageError::ResourceTenantMismatch);
    }
    if bucket_id.kind_label().map_err(map_resource_error)? != "bucket" {
        return Err(CloudStorageError::ResourceKindMismatch);
    }
    Ok(bucket_id)
}

fn validate_object_key_prefix(
    prefix: &ObjectKey,
    tenant_id: &str,
    cell_id: &CellId,
) -> Result<(), CloudStorageError> {
    let expected_prefix = format!("{}/{}/", tenant_id, cell_id.value);
    if prefix.value != expected_prefix && !prefix.value.starts_with(&expected_prefix) {
        return Err(CloudStorageError::InvalidStorageNamespacePolicy);
    }
    if prefix.value.contains("//") {
        return Err(CloudStorageError::InvalidStorageNamespacePolicy);
    }
    Ok(())
}

fn validate_metadata_refs(
    values: Vec<String>,
    prefix: &str,
) -> Result<Vec<String>, CloudStorageError> {
    if values.is_empty() {
        return Err(CloudStorageError::InvalidEvidenceRef);
    }
    let mut seen = BTreeSet::new();
    let mut refs = Vec::with_capacity(values.len());
    for value in values {
        let value = validate_metadata_ref(value, prefix)?;
        if !seen.insert(value.clone()) {
            return Err(CloudStorageError::InvalidEvidenceRef);
        }
        refs.push(value);
    }
    Ok(refs)
}

fn validate_metadata_ref(value: String, prefix: &str) -> Result<String, CloudStorageError> {
    let trimmed = value.trim();
    if trimmed != value {
        return Err(CloudStorageError::InvalidEvidenceRef);
    }
    if !value.starts_with(prefix)
        || value.len() <= prefix.len()
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        return Err(CloudStorageError::InvalidEvidenceRef);
    }
    if looks_secret_like(&value) {
        return Err(CloudStorageError::InvalidEvidenceRef);
    }
    Ok(value)
}

fn looks_secret_like(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "token=",
        "password",
        "secret",
        "credential",
        "private_key",
        "private-key",
        "api_key",
        "apikey",
        "-----begin",
        "sk-live",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

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

fn validate_time_order(start: u64, end: u64) -> Result<(), CloudStorageError> {
    if end >= start {
        Ok(())
    } else {
        Err(CloudStorageError::InvalidTimeOrder)
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

fn validate_dns_name(
    value: &str,
    max_len: usize,
    error: CloudStorageError,
) -> Result<(), CloudStorageError> {
    if value.len() < 3
        || value.len() > max_len
        || value.starts_with('-')
        || value.ends_with('-')
        || value.contains("..")
        || value.contains("--")
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'.'
        })
    {
        return Err(error);
    }
    Ok(())
}

fn canonical_name(value: String, error: CloudStorageError) -> Result<String, CloudStorageError> {
    validate_canonical_segment(&value, error)?;
    Ok(value)
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
        return Err(error);
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

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}
