fn cloud_storage_status_kind(error: &CloudStorageError) -> CloudStorageObjectApiStatusKind {
    match error {
        CloudStorageError::DuplicateBucket
        | CloudStorageError::DuplicateObject
        | CloudStorageError::DuplicateVolume
        | CloudStorageError::DuplicateFilesystem
        | CloudStorageError::DuplicateArchiveVault
        | CloudStorageError::DuplicateSnapshot => CloudStorageObjectApiStatusKind::Conflict,
        CloudStorageError::UnknownBucket | CloudStorageError::UnknownVolume => {
            CloudStorageObjectApiStatusKind::NotFound
        }
        CloudStorageError::ResourceTenantMismatch
        | CloudStorageError::ResourceRegionMismatch
        | CloudStorageError::KmsKeyModeMismatch
        | CloudStorageError::KmsKeyTenantMismatch
        | CloudStorageError::KmsKeyRegionMismatch
        | CloudStorageError::ReplicationResidencyDenied
        | CloudStorageError::ObjectDataClassDenied
        | CloudStorageError::CellLocationMismatch => CloudStorageObjectApiStatusKind::Forbidden,
        CloudStorageError::InvalidTenantId
        | CloudStorageError::InvalidResourceId
        | CloudStorageError::ResourceKindMismatch
        | CloudStorageError::InvalidBucketName
        | CloudStorageError::InvalidObjectKey
        | CloudStorageError::InvalidEtag
        | CloudStorageError::InvalidKmsKeyId
        | CloudStorageError::InvalidKmsKeyVersion
        | CloudStorageError::InvalidKmsUseEventId
        | CloudStorageError::InvalidMaterialRef
        | CloudStorageError::InvalidCiphertextRef
        | CloudStorageError::InvalidKmsPurpose
        | CloudStorageError::InvalidDestructionProofRef
        | CloudStorageError::MissingKmsKey
        | CloudStorageError::UnexpectedKmsKey
        | CloudStorageError::InvalidReplicationPolicy
        | CloudStorageError::DuplicateReplicationRegion
        | CloudStorageError::EmptyAllowedDataClassSet
        | CloudStorageError::DuplicateDataClass
        | CloudStorageError::InvalidDataClass
        | CloudStorageError::InvalidObjectLockPolicy
        | CloudStorageError::InvalidSize
        | CloudStorageError::InvalidPerformance
        | CloudStorageError::InvalidAzCode
        | CloudStorageError::InvalidCellId
        | CloudStorageError::AzRegionMismatch
        | CloudStorageError::InvalidSnapshotId
        | CloudStorageError::InvalidInitialState
        | CloudStorageError::InvalidTimeOrder
        | CloudStorageError::InvalidStorageNamespacePolicy
        | CloudStorageError::InvalidEvidenceRef => CloudStorageObjectApiStatusKind::BadRequest,
    }
}

fn cloud_storage_message(error: &CloudStorageError) -> &'static str {
    match cloud_storage_status_kind(error) {
        CloudStorageObjectApiStatusKind::BadRequest => "Cloud Storage rejected the request shape",
        CloudStorageObjectApiStatusKind::Forbidden => "Cloud Storage policy denied the request",
        CloudStorageObjectApiStatusKind::NotFound => "Cloud Storage resource was not found",
        CloudStorageObjectApiStatusKind::Conflict => "Cloud Storage resource already exists",
        CloudStorageObjectApiStatusKind::UnprocessableEntity => {
            "Cloud Storage rejected request idempotency"
        }
    }
}

fn cloud_storage_issue(error: &CloudStorageError) -> &'static str {
    match error {
        CloudStorageError::InvalidTenantId => "tenant_id must be a ten_ identifier",
        CloudStorageError::InvalidResourceId => "resource_id must be canonical cloud resource id",
        CloudStorageError::ResourceTenantMismatch => "resource tenant must match request tenant",
        CloudStorageError::ResourceRegionMismatch => "resource region must match request region",
        CloudStorageError::ResourceKindMismatch => "resource kind must match storage type",
        CloudStorageError::InvalidBucketName => "bucket name must be canonical DNS label",
        CloudStorageError::InvalidObjectKey => "object key must be non-empty and bounded",
        CloudStorageError::InvalidEtag => "etag must be a canonical checksum value",
        CloudStorageError::InvalidKmsKeyId => "kms_key must be canonical for the selected mode",
        CloudStorageError::InvalidKmsKeyVersion => "kms key version must be greater than zero",
        CloudStorageError::InvalidKmsUseEventId => "kms use event id must be canonical",
        CloudStorageError::InvalidMaterialRef => "material_ref must be a matref/ reference",
        CloudStorageError::InvalidCiphertextRef => "ciphertext_ref must be a ct/ reference",
        CloudStorageError::InvalidKmsPurpose => "KMS purpose must match the storage surface",
        CloudStorageError::InvalidDestructionProofRef => "destruction proof must be canonical",
        CloudStorageError::MissingKmsKey => "selected encryption mode requires kms_key",
        CloudStorageError::UnexpectedKmsKey => "selected encryption mode does not accept kms_key",
        CloudStorageError::KmsKeyModeMismatch => {
            "kms_key origin must match selected encryption mode"
        }
        CloudStorageError::KmsKeyTenantMismatch => "kms_key tenant must match request tenant",
        CloudStorageError::KmsKeyRegionMismatch => "kms_key region must match request region",
        CloudStorageError::InvalidReplicationPolicy => "replication policy must be canonical",
        CloudStorageError::DuplicateReplicationRegion => "replication destinations must be unique",
        CloudStorageError::ReplicationResidencyDenied => {
            "replication must satisfy residency policy"
        }
        CloudStorageError::EmptyAllowedDataClassSet => "allowed data-class set must not be empty",
        CloudStorageError::DuplicateDataClass => "allowed data classes must be unique",
        CloudStorageError::InvalidDataClass => "data_class must be a privacy-program class",
        CloudStorageError::ObjectDataClassDenied => {
            "object data_class must be admitted by bucket policy"
        }
        CloudStorageError::InvalidObjectLockPolicy => {
            "object lock policy must define retention or hold"
        }
        CloudStorageError::InvalidSize => "size must be greater than zero",
        CloudStorageError::InvalidPerformance => "volume performance must be greater than zero",
        CloudStorageError::InvalidAzCode => "AZ must be canonical lowercase ASCII",
        CloudStorageError::InvalidCellId => "cell_id must be canonical and use the cell- prefix",
        CloudStorageError::AzRegionMismatch => "AZ code must sit under its region code",
        CloudStorageError::CellLocationMismatch => "cell_id must sit under its AZ and region",
        CloudStorageError::InvalidSnapshotId => "snapshot id must use the snap_ prefix",
        CloudStorageError::InvalidInitialState => {
            "create requests must start in active storage state"
        }
        CloudStorageError::InvalidTimeOrder => "request timestamps must be monotonic",
        CloudStorageError::InvalidStorageNamespacePolicy => {
            "tenant/cell storage namespace policy must be canonical"
        }
        CloudStorageError::InvalidEvidenceRef => {
            "evidence refs must be canonical and must not contain credentials"
        }
        CloudStorageError::DuplicateBucket => "bucket resource id is already present",
        CloudStorageError::UnknownBucket => "bucket must exist before object creation",
        CloudStorageError::DuplicateObject => "object key is already present in the bucket",
        CloudStorageError::DuplicateVolume => "volume resource id is already present",
        CloudStorageError::UnknownVolume => "volume must exist before snapshot creation",
        CloudStorageError::DuplicateFilesystem => "filesystem resource id is already present",
        CloudStorageError::DuplicateArchiveVault => "archive vault resource id is already present",
        CloudStorageError::DuplicateSnapshot => "snapshot id is already present",
    }
}

fn detail(field: &str, issue: &str) -> CloudStorageObjectApiErrorDetail {
    CloudStorageObjectApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
