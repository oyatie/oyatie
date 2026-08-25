use super::*;
use storage_provider_draft::{CloudStorageError, DataClass, StorageProviderObjectPort};

const REGION: &str = "ap-northeast-2";
const BUCKET: &str = "oyatie-s3-cold-backup";
const BUCKET_ID: &str = "oyatie:cloud:alpha-region:ten_alpha:bucket:tenant-assets";
const OBJECT_KEY: &str = "workspace/report.pdf";

fn adapter() -> S3Adapter {
    S3Adapter::new("https://s3.ap-northeast-2.amazonaws.com", REGION, BUCKET)
        .unwrap()
        .with_clock(1_700_000_000)
}

fn put_request() -> StorageProviderObjectPutRequest {
    StorageProviderObjectPutRequest {
        request_id: "storageprov_req_put_001".to_string(),
        provider_bucket_ref: format!("s3://{REGION}/{BUCKET}"),
        bucket_id: BUCKET_ID.to_string(),
        tenant_id: "ten_alpha".to_string(),
        object_key: OBJECT_KEY.to_string(),
        object_body_ref: "objbody/ten_alpha/workspace/report".to_string(),
        size_bytes: 42,
        etag: "0123456789abcdef0123456789abcdef".to_string(),
        data_class: DataClass::PiiIdentifying,
        kms_key: "kms/alpha-region/ten_alpha/object-key".to_string(),
        ciphertext_ref: "ct/ten_alpha/object/report".to_string(),
        actor: "sp_storage".to_string(),
        idempotency_key: "idem-storage-object-put".to_string(),
        requested_at_epoch_seconds: 1_700_000_010,
    }
}

fn get_request() -> StorageProviderObjectGetRequest {
    StorageProviderObjectGetRequest {
        request_id: "storageprov_req_get_001".to_string(),
        provider_bucket_ref: format!("s3://{REGION}/{BUCKET}"),
        bucket_id: BUCKET_ID.to_string(),
        tenant_id: "ten_alpha".to_string(),
        object_key: OBJECT_KEY.to_string(),
        result_body_ref: "objbody/ten_alpha/workspace/report-read".to_string(),
        actor: "sp_storage".to_string(),
        requested_at_epoch_seconds: 1_700_000_020,
    }
}

#[test]
fn put_command_uses_s3_object_path_and_reference_only_body() {
    let command = adapter()
        .put_command(&put_request())
        .expect("valid put request becomes deterministic S3 command");

    assert_eq!(command.operation, "PutObject");
    assert_eq!(command.method, "PUT");
    assert_eq!(
        command.endpoint_origin,
        "https://s3.ap-northeast-2.amazonaws.com"
    );
    assert_eq!(command.path, format!("/{BUCKET}/workspace%2Freport.pdf"));
    assert!(command.body_canonical.contains("object_body_ref=objbody/"));
    assert!(
        command
            .body_canonical
            .contains("data_class=PII_IDENTIFYING")
    );
    assert!(!command.body_canonical.contains("raw_bytes"));
    assert!(!command.body_canonical.contains("secret_access_key"));
    assert_eq!(
        command.provider_evidence_ref,
        format!("s3://{REGION}/{BUCKET}/{OBJECT_KEY}/storageprov_req_put_001")
    );
}

#[test]
fn put_command_percent_encodes_all_non_unreserved_path_bytes() {
    let mut request = put_request();
    request.object_key = "workspace/final report #1.pdf".to_string();
    let command = adapter()
        .put_command(&request)
        .expect("valid put request encodes S3 object path");

    assert_eq!(
        command.path,
        format!("/{BUCKET}/workspace%2Ffinal%20report%20%231.pdf")
    );
}

#[test]
fn get_command_uses_result_reference_without_payload_bytes() {
    let command = adapter()
        .get_command(&get_request())
        .expect("valid get request becomes deterministic S3 command");

    assert_eq!(command.operation, "GetObject");
    assert_eq!(command.method, "GET");
    assert!(command.body_canonical.contains("result_body_ref=objbody/"));
    assert!(!command.body_canonical.contains("raw_bytes"));
    assert!(!command.body_canonical.contains("secret_access_key"));
}

#[test]
fn s3_object_port_receipts_redact_provider_payloads() {
    let adapter = adapter();
    let put = adapter
        .put_object(put_request())
        .expect("put receipt is generated");
    let get = adapter
        .get_object(get_request())
        .expect("get receipt is generated");

    assert_eq!(put.provider, StorageProviderKind::S3ObjectStorage);
    assert_eq!(put.provider.label(), "s3_object_storage");
    assert_eq!(
        put.provider_request_id,
        "s3-object-1700000000-storageprov_req_put_001"
    );
    assert_eq!(put.object_body_ref, "objbody/ten_alpha/workspace/report");
    assert_eq!(put.size_bytes, Some(42));
    assert_eq!(
        get.object_body_ref,
        "objbody/ten_alpha/workspace/report-read"
    );
    assert_eq!(get.size_bytes, None);
}

#[test]
fn rejects_provider_bucket_drift_and_bad_bucket_shape() {
    let mut drifted = put_request();
    drifted.provider_bucket_ref = "s3://other/bucket".to_string();
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
fn rejects_invalid_s3_adapter_config() {
    assert_eq!(
        S3Adapter::new("http://s3.ap-northeast-2.amazonaws.com", REGION, BUCKET),
        Err(S3AdapterConfigError::InvalidEndpoint)
    );
    assert_eq!(
        S3Adapter::new(
            "https://s3.ap-northeast-2.amazonaws.com",
            "AP Northeast",
            BUCKET
        ),
        Err(S3AdapterConfigError::InvalidRegion)
    );
    assert_eq!(
        S3Adapter::new(
            "https://s3.ap-northeast-2.amazonaws.com",
            REGION,
            "bad_bucket"
        ),
        Err(S3AdapterConfigError::InvalidBucketName)
    );
}
