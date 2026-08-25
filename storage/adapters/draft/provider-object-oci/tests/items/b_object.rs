#[test]
fn rejects_provider_bucket_drift_and_bad_bucket_shape() {
    let mut drifted = put_request();
    drifted.provider_bucket_ref = "oci-object://other/bucket".to_string();
    assert!(matches!(
        adapter().put_command(&drifted),
        Err(StorageProviderObjectError::ProviderRejected { .. })
    ));

    let mut bad_bucket = put_request();
    bad_bucket.bucket_id = "oyatie:cloud:alpha-region:ten_alpha:volume:not-bucket".to_string();
    assert_eq!(
        bad_bucket.validate(),
        Err(StorageProviderObjectError::InvalidRequestShape(
            CloudStorageError::ResourceKindMismatch,
        ))
    );
}

#[test]
fn rejects_invalid_object_adapter_config() {
    assert_eq!(
        OciObjectStorageAdapter::new("http://objectstorage", NAMESPACE, BUCKET),
        Err(OciObjectStorageAdapterConfigError::InvalidEndpoint)
    );
    assert_eq!(
        OciObjectStorageAdapter::new(
            "https://objectstorage.ap-chuncheon-1.oraclecloud.com",
            "bad namespace",
            BUCKET,
        ),
        Err(OciObjectStorageAdapterConfigError::InvalidNamespace)
    );
}
