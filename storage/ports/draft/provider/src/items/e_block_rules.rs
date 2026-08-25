impl StorageProviderBlockCreateVolumeRequest {
    pub fn validate(&self) -> Result<(), StorageProviderBlockError> {
        validate_provider_block_ref(
            &self.request_id,
            StorageProviderBlockError::InvalidProviderRequestId,
        )?;
        validate_provider_block_ref(
            &self.provider_volume_ref,
            StorageProviderBlockError::InvalidProviderVolumeRef,
        )?;
        validate_provider_block_ref(
            &self.idempotency_key,
            StorageProviderBlockError::InvalidIdempotencyKey,
        )?;
        validate_block_request(self).map_err(StorageProviderBlockError::InvalidRequestShape)?;
        PrincipalId::new(self.actor.clone())
            .map_err(|_| StorageProviderBlockError::InvalidActorRef)?;
        Ok(())
    }
}

impl StorageProviderBlockReceipt {
    pub fn create_volume(
        provider: StorageProviderKind,
        input: StorageProviderBlockCreateVolumeRequest,
        provider_request_id: impl Into<String>,
        provider_evidence_ref: impl Into<String>,
    ) -> Result<Self, StorageProviderBlockError> {
        input.validate()?;
        let provider_request_id = provider_request_id.into();
        let provider_evidence_ref = provider_evidence_ref.into();
        validate_provider_block_ref(
            &provider_request_id,
            StorageProviderBlockError::InvalidProviderRequestId,
        )?;
        validate_provider_block_ref(
            &provider_evidence_ref,
            StorageProviderBlockError::InvalidProviderEvidenceRef,
        )?;
        Ok(Self {
            provider,
            operation: StorageBlockOperation::CreateVolume,
            request_id: input.request_id,
            provider_request_id,
            provider_volume_ref: input.provider_volume_ref,
            volume_id: input.volume_id,
            tenant_id: input.tenant_id,
            name: input.name,
            region: input.region,
            az: input.az,
            cell_id: input.cell_id,
            residency: input.residency,
            tier: input.tier,
            size_gib: input.size_gib,
            performance: input.performance,
            encryption: input.encryption,
            kms_key: input.kms_key,
            data_class: input.data_class,
            actor: input.actor,
            idempotency_key: input.idempotency_key,
            provider_evidence_ref,
            occurred_at_epoch_seconds: input.requested_at_epoch_seconds,
            schema_version: PROVIDER_SCHEMA_VERSION,
        })
    }
}

fn validate_block_request(
    input: &StorageProviderBlockCreateVolumeRequest,
) -> Result<(), CloudStorageError> {
    validate_tenant_id(&input.tenant_id)?;
    let region =
        RegionCode::new(input.region.clone()).map_err(|_| CloudStorageError::InvalidResourceId)?;
    let az = AzCode::new(input.az.clone()).map_err(|_| CloudStorageError::InvalidAzCode)?;
    let cell_id =
        CellId::new(input.cell_id.clone()).map_err(|_| CloudStorageError::InvalidCellId)?;
    validate_az_region(&az, &region)?;
    validate_cell_location(&cell_id, &region, Some(&az))?;
    validate_residency_allows_region(&input.residency, &region)?;
    validate_size(input.size_gib)?;
    validate_performance(input.performance)?;
    resource_id_for(
        &input.volume_id,
        &input.tenant_id,
        &region,
        ResourceKind::Volume(input.tier),
    )?;
    validate_encryption_key(
        input.encryption,
        input.kms_key.as_deref(),
        &region,
        &input.tenant_id,
    )?;
    validate_canonical_segment(&input.name, CloudStorageError::InvalidResourceId)?;
    privacy_class(input.data_class)?;
    Ok(())
}
