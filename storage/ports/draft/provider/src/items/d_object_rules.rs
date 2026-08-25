impl StorageProviderObjectPutRequest {
    pub fn validate(&self) -> Result<(), StorageProviderObjectError> {
        validate_provider_ref(
            &self.request_id,
            StorageProviderObjectError::InvalidProviderRequestId,
        )?;
        validate_provider_ref(
            &self.provider_bucket_ref,
            StorageProviderObjectError::InvalidProviderBucketRef,
        )?;
        validate_provider_ref(
            &self.object_body_ref,
            StorageProviderObjectError::InvalidObjectBodyRef,
        )?;
        validate_provider_ref(
            &self.idempotency_key,
            StorageProviderObjectError::InvalidIdempotencyKey,
        )?;
        validate_bucket_resource(&self.bucket_id, &self.tenant_id)
            .map_err(StorageProviderObjectError::InvalidRequestShape)?;
        validate_object_key(&self.object_key)
            .map_err(StorageProviderObjectError::InvalidRequestShape)?;
        validate_tenant_id(&self.tenant_id)
            .map_err(StorageProviderObjectError::InvalidRequestShape)?;
        validate_size(self.size_bytes).map_err(StorageProviderObjectError::InvalidRequestShape)?;
        validate_etag(&self.etag).map_err(StorageProviderObjectError::InvalidRequestShape)?;
        privacy_class(self.data_class).map_err(StorageProviderObjectError::InvalidRequestShape)?;
        KmsKeyId::new(self.kms_key.clone()).map_err(|_| {
            StorageProviderObjectError::InvalidRequestShape(CloudStorageError::InvalidKmsKeyId)
        })?;
        CiphertextRef::new(self.ciphertext_ref.clone()).map_err(|_| {
            StorageProviderObjectError::InvalidRequestShape(CloudStorageError::InvalidCiphertextRef)
        })?;
        PrincipalId::new(self.actor.clone())
            .map_err(|_| StorageProviderObjectError::InvalidActorRef)?;
        Ok(())
    }
}

impl StorageProviderObjectGetRequest {
    pub fn validate(&self) -> Result<(), StorageProviderObjectError> {
        validate_provider_ref(
            &self.request_id,
            StorageProviderObjectError::InvalidProviderRequestId,
        )?;
        validate_provider_ref(
            &self.provider_bucket_ref,
            StorageProviderObjectError::InvalidProviderBucketRef,
        )?;
        validate_provider_ref(
            &self.result_body_ref,
            StorageProviderObjectError::InvalidObjectBodyRef,
        )?;
        validate_bucket_resource(&self.bucket_id, &self.tenant_id)
            .map_err(StorageProviderObjectError::InvalidRequestShape)?;
        validate_object_key(&self.object_key)
            .map_err(StorageProviderObjectError::InvalidRequestShape)?;
        validate_tenant_id(&self.tenant_id)
            .map_err(StorageProviderObjectError::InvalidRequestShape)?;
        PrincipalId::new(self.actor.clone())
            .map_err(|_| StorageProviderObjectError::InvalidActorRef)?;
        Ok(())
    }
}

impl StorageProviderObjectReceipt {
    pub fn put_object(
        provider: StorageProviderKind,
        input: StorageProviderObjectPutRequest,
        provider_request_id: impl Into<String>,
        provider_evidence_ref: impl Into<String>,
    ) -> Result<Self, StorageProviderObjectError> {
        input.validate()?;
        let provider_request_id = provider_request_id.into();
        let provider_evidence_ref = provider_evidence_ref.into();
        validate_provider_ref(
            &provider_request_id,
            StorageProviderObjectError::InvalidProviderRequestId,
        )?;
        validate_provider_ref(
            &provider_evidence_ref,
            StorageProviderObjectError::InvalidProviderEvidenceRef,
        )?;
        Ok(Self {
            provider,
            operation: StorageObjectOperation::PutObject,
            request_id: input.request_id,
            provider_request_id,
            provider_bucket_ref: input.provider_bucket_ref,
            bucket_id: input.bucket_id,
            tenant_id: input.tenant_id,
            object_key: input.object_key,
            object_body_ref: input.object_body_ref,
            size_bytes: Some(input.size_bytes),
            etag: Some(input.etag),
            data_class: Some(input.data_class),
            kms_key: Some(input.kms_key),
            ciphertext_ref: Some(input.ciphertext_ref),
            actor: input.actor,
            provider_evidence_ref,
            occurred_at_epoch_seconds: input.requested_at_epoch_seconds,
            schema_version: PROVIDER_SCHEMA_VERSION,
        })
    }

    pub fn get_object(
        provider: StorageProviderKind,
        input: StorageProviderObjectGetRequest,
        provider_request_id: impl Into<String>,
        provider_evidence_ref: impl Into<String>,
    ) -> Result<Self, StorageProviderObjectError> {
        input.validate()?;
        let provider_request_id = provider_request_id.into();
        let provider_evidence_ref = provider_evidence_ref.into();
        validate_provider_ref(
            &provider_request_id,
            StorageProviderObjectError::InvalidProviderRequestId,
        )?;
        validate_provider_ref(
            &provider_evidence_ref,
            StorageProviderObjectError::InvalidProviderEvidenceRef,
        )?;
        Ok(Self {
            provider,
            operation: StorageObjectOperation::GetObject,
            request_id: input.request_id,
            provider_request_id,
            provider_bucket_ref: input.provider_bucket_ref,
            bucket_id: input.bucket_id,
            tenant_id: input.tenant_id,
            object_key: input.object_key,
            object_body_ref: input.result_body_ref,
            size_bytes: None,
            etag: None,
            data_class: None,
            kms_key: None,
            ciphertext_ref: None,
            actor: input.actor,
            provider_evidence_ref,
            occurred_at_epoch_seconds: input.requested_at_epoch_seconds,
            schema_version: PROVIDER_SCHEMA_VERSION,
        })
    }
}
