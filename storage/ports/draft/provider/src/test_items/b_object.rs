#[test]
fn object_requests_preserve_validation_contract() {
    object_put_request()
        .validate()
        .expect("put request is valid");
    object_get_request()
        .validate()
        .expect("get request is valid");

    let mut wrong_kind = object_put_request();
    wrong_kind.bucket_id = "oyatie:cloud:region-alpha1:ten_alpha:volume:not-bucket".to_string();
    assert_eq!(
        wrong_kind.validate(),
        Err(StorageProviderObjectError::InvalidRequestShape(
            CloudStorageError::ResourceKindMismatch,
        ))
    );

    let mut bad_actor = object_get_request();
    bad_actor.actor = "storage".to_string();
    assert_eq!(
        bad_actor.validate(),
        Err(StorageProviderObjectError::InvalidActorRef)
    );
}

#[test]
fn object_receipts_keep_references_without_payload_bytes() {
    let put = StorageProviderObjectReceipt::put_object(
        StorageProviderKind::S3ObjectStorage,
        object_put_request(),
        "s3-put-001",
        "s3://tenant-assets/workspace/report.pdf/put",
    )
    .expect("put receipt is valid");
    let get = StorageProviderObjectReceipt::get_object(
        StorageProviderKind::S3ObjectStorage,
        object_get_request(),
        "s3-get-001",
        "s3://tenant-assets/workspace/report.pdf/get",
    )
    .expect("get receipt is valid");

    assert_eq!(put.operation, StorageObjectOperation::PutObject);
    assert_eq!(put.size_bytes, Some(42));
    assert_eq!(put.object_body_ref, "objbody/ten_alpha/workspace/report");
    assert_eq!(get.operation, StorageObjectOperation::GetObject);
    assert_eq!(get.size_bytes, None);
    assert_eq!(
        get.object_body_ref,
        "objbody/ten_alpha/workspace/report-read"
    );
}

#[test]
fn object_requests_reject_operational_data_classes() {
    let mut request = object_put_request();
    request.data_class = DataClass::Audit;
    assert_eq!(
        request.validate(),
        Err(StorageProviderObjectError::InvalidRequestShape(
            CloudStorageError::InvalidDataClass,
        ))
    );
}
