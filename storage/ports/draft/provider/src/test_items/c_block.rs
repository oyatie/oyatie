#[test]
fn block_requests_preserve_validation_contract() {
    block_create_request()
        .validate()
        .expect("block request is valid");

    let mut wrong_kind = block_create_request();
    wrong_kind.volume_id = "oyatie:cloud:region-alpha1:ten_alpha:bucket:not-volume".to_string();
    assert_eq!(
        wrong_kind.validate(),
        Err(StorageProviderBlockError::InvalidRequestShape(
            CloudStorageError::ResourceKindMismatch,
        ))
    );

    let mut bad_actor = block_create_request();
    bad_actor.actor = "storage".to_string();
    assert_eq!(
        bad_actor.validate(),
        Err(StorageProviderBlockError::InvalidActorRef)
    );
}

#[test]
fn block_validation_keeps_location_and_encryption_checks() {
    let mut wrong_cell = block_create_request();
    wrong_cell.cell_id = "cell-region-beta1-a-001".to_string();
    assert_eq!(
        wrong_cell.validate(),
        Err(StorageProviderBlockError::InvalidRequestShape(
            CloudStorageError::CellLocationMismatch,
        ))
    );

    let mut wrong_key = block_create_request();
    wrong_key.kms_key = Some("kms/region-alpha1/ten_alpha/db-key".to_string());
    assert_eq!(
        wrong_key.validate(),
        Err(StorageProviderBlockError::InvalidRequestShape(
            CloudStorageError::KmsKeyModeMismatch,
        ))
    );
}

#[test]
fn block_receipts_keep_refs_without_credentials() {
    let receipt = StorageProviderBlockReceipt::create_volume(
        StorageProviderKind::OciBlockStorage,
        block_create_request(),
        "oci-block-create-001",
        "oci-block://compartment/region-alpha1-a/db-primary/create",
    )
    .expect("block receipt is valid");

    assert_eq!(receipt.operation, StorageBlockOperation::CreateVolume);
    assert_eq!(receipt.size_gib, 512);
    assert_eq!(receipt.performance.iops, 12_000);
    assert_eq!(receipt.encryption, EncryptionMode::Byok);
    assert_eq!(receipt.schema_version, PROVIDER_SCHEMA_VERSION);
}
