//! OCI Object Storage adapter boundary for Cloud Storage.
//!
//! This crate keeps OCI namespace, bucket, object path, and evidence refs outside
//! provider-neutral Cloud Storage domain/API crates while implementing the shared
//! `StorageProviderObjectPort` contract. It builds deterministic request shapes;
//! credentialed live smoke remains a separate promotion gate.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_cloud_storage_domain::{
    StorageProviderKind, StorageProviderObjectError, StorageProviderObjectGetRequest,
    StorageProviderObjectPort, StorageProviderObjectPutRequest, StorageProviderObjectReceipt,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OciObjectStorageAdapterConfigError {
    InvalidEndpoint,
    InvalidNamespace,
    InvalidBucketName,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciObjectStorageAdapter {
    endpoint_origin: String,  // data_class: INTERNAL_ONLY
    namespace_name: String,   // data_class: INTERNAL_ONLY
    bucket_name: String,      // data_class: INTERNAL_ONLY
    clock_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciObjectStorageCommand {
    pub operation: &'static str,       // data_class: PUBLIC
    pub method: &'static str,          // data_class: PUBLIC
    pub endpoint_origin: String,       // data_class: INTERNAL_ONLY
    pub path: String,                  // data_class: INTERNAL_ONLY
    pub body_canonical: String,        // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String, // data_class: INTERNAL_ONLY
}

impl OciObjectStorageAdapter {
    pub fn new(
        endpoint_origin: impl Into<String>,
        namespace_name: impl Into<String>,
        bucket_name: impl Into<String>,
    ) -> Result<Self, OciObjectStorageAdapterConfigError> {
        let endpoint_origin = endpoint_origin.into();
        let namespace_name = namespace_name.into();
        let bucket_name = bucket_name.into();
        validate_endpoint(&endpoint_origin)?;
        validate_segment(
            &namespace_name,
            OciObjectStorageAdapterConfigError::InvalidNamespace,
        )?;
        validate_segment(
            &bucket_name,
            OciObjectStorageAdapterConfigError::InvalidBucketName,
        )?;
        Ok(Self {
            endpoint_origin,
            namespace_name,
            bucket_name,
            clock_epoch_seconds: 0,
        })
    }

    pub fn with_clock(mut self, clock_epoch_seconds: u64) -> Self {
        self.clock_epoch_seconds = clock_epoch_seconds;
        self
    }

    pub fn provider_bucket_ref(&self) -> String {
        format!("oci-object://{}/{}", self.namespace_name, self.bucket_name)
    }

    pub fn put_command(
        &self,
        request: &StorageProviderObjectPutRequest,
    ) -> Result<OciObjectStorageCommand, StorageProviderObjectError> {
        request.validate()?;
        self.ensure_provider_bucket(&request.provider_bucket_ref)?;
        let size_bytes = request.size_bytes.to_string();
        Ok(self.command(
            "PutObject",
            "PUT",
            &request.object_key,
            &request.request_id,
            &[
                ("bucket_id", request.bucket_id.as_str()),
                ("tenant_id", request.tenant_id.as_str()),
                ("object_key", request.object_key.as_str()),
                ("object_body_ref", request.object_body_ref.as_str()),
                ("size_bytes", size_bytes.as_str()),
                ("etag", request.etag.as_str()),
                ("data_class", request.data_class.label()),
                ("kms_key", request.kms_key.as_str()),
                ("ciphertext_ref", request.ciphertext_ref.as_str()),
                ("actor", request.actor.as_str()),
                ("idempotency_key", request.idempotency_key.as_str()),
            ],
        ))
    }

    pub fn get_command(
        &self,
        request: &StorageProviderObjectGetRequest,
    ) -> Result<OciObjectStorageCommand, StorageProviderObjectError> {
        request.validate()?;
        self.ensure_provider_bucket(&request.provider_bucket_ref)?;
        Ok(self.command(
            "GetObject",
            "GET",
            &request.object_key,
            &request.request_id,
            &[
                ("bucket_id", request.bucket_id.as_str()),
                ("tenant_id", request.tenant_id.as_str()),
                ("object_key", request.object_key.as_str()),
                ("result_body_ref", request.result_body_ref.as_str()),
                ("actor", request.actor.as_str()),
            ],
        ))
    }

    fn command(
        &self,
        operation: &'static str,
        method: &'static str,
        object_key: &str,
        request_id: &str,
        fields: &[(&str, &str)],
    ) -> OciObjectStorageCommand {
        OciObjectStorageCommand {
            operation,
            method,
            endpoint_origin: self.endpoint_origin.clone(),
            path: format!(
                "/n/{}/b/{}/o/{}",
                self.namespace_name,
                self.bucket_name,
                encode_object_path(object_key)
            ),
            body_canonical: canonical_body(fields),
            provider_evidence_ref: format!(
                "oci-object://{}/{}/{}/{}",
                self.namespace_name, self.bucket_name, object_key, request_id
            ),
        }
    }

    fn ensure_provider_bucket(
        &self,
        provider_bucket_ref: &str,
    ) -> Result<(), StorageProviderObjectError> {
        let expected = self.provider_bucket_ref();
        if provider_bucket_ref == expected {
            Ok(())
        } else {
            Err(StorageProviderObjectError::ProviderRejected {
                provider: StorageProviderKind::OciObjectStorage,
                reason: "provider_bucket_ref does not match configured OCI Object Storage bucket"
                    .to_string(),
            })
        }
    }

    fn provider_request_id(&self, request_id: &str) -> String {
        format!("oci-object-{}-{request_id}", self.clock_epoch_seconds)
    }
}

impl StorageProviderObjectPort for OciObjectStorageAdapter {
    fn provider_kind(&self) -> StorageProviderKind {
        StorageProviderKind::OciObjectStorage
    }

    fn put_object(
        &self,
        input: StorageProviderObjectPutRequest,
    ) -> Result<StorageProviderObjectReceipt, StorageProviderObjectError> {
        let command = self.put_command(&input)?;
        StorageProviderObjectReceipt::put_object(
            self.provider_kind(),
            input.clone(),
            self.provider_request_id(&input.request_id),
            command.provider_evidence_ref,
        )
    }

    fn get_object(
        &self,
        input: StorageProviderObjectGetRequest,
    ) -> Result<StorageProviderObjectReceipt, StorageProviderObjectError> {
        let command = self.get_command(&input)?;
        StorageProviderObjectReceipt::get_object(
            self.provider_kind(),
            input.clone(),
            self.provider_request_id(&input.request_id),
            command.provider_evidence_ref,
        )
    }
}

fn validate_endpoint(value: &str) -> Result<(), OciObjectStorageAdapterConfigError> {
    if value.starts_with("https://") && no_space_or_control(value) {
        Ok(())
    } else {
        Err(OciObjectStorageAdapterConfigError::InvalidEndpoint)
    }
}

fn validate_segment(
    value: &str,
    error: OciObjectStorageAdapterConfigError,
) -> Result<(), OciObjectStorageAdapterConfigError> {
    if value.trim().is_empty() || value.contains('/') || !no_space_or_control(value) {
        Err(error)
    } else {
        Ok(())
    }
}

fn no_space_or_control(value: &str) -> bool {
    !value
        .bytes()
        .any(|byte| byte.is_ascii_control() || byte == b' ')
}

fn encode_object_path(object_key: &str) -> String {
    object_key.replace('/', "%2F")
}

fn canonical_body(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_cloud_storage_domain::CloudStorageError;
    use oya_data_boundary_kernel::DataClass;

    const NAMESPACE: &str = "axdotp9iv3ua";
    const BUCKET: &str = "oyatie-audit-cold-backup";
    const BUCKET_ID: &str = "oya:cloud:alpha-region:ten_alpha:bucket:tenant-assets";
    const OBJECT_KEY: &str = "workspace/report.pdf";

    fn adapter() -> OciObjectStorageAdapter {
        OciObjectStorageAdapter::new(
            "https://objectstorage.ap-chuncheon-1.oraclecloud.com",
            NAMESPACE,
            BUCKET,
        )
        .unwrap()
        .with_clock(1_700_000_000)
    }

    fn put_request() -> StorageProviderObjectPutRequest {
        StorageProviderObjectPutRequest {
            request_id: "storageprov_req_put_001".to_string(),
            provider_bucket_ref: format!("oci-object://{NAMESPACE}/{BUCKET}"),
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
            provider_bucket_ref: format!("oci-object://{NAMESPACE}/{BUCKET}"),
            bucket_id: BUCKET_ID.to_string(),
            tenant_id: "ten_alpha".to_string(),
            object_key: OBJECT_KEY.to_string(),
            result_body_ref: "objbody/ten_alpha/workspace/report-read".to_string(),
            actor: "sp_storage".to_string(),
            requested_at_epoch_seconds: 1_700_000_020,
        }
    }

    #[test]
    fn put_command_uses_oci_object_path_and_reference_only_body() {
        let command = adapter()
            .put_command(&put_request())
            .expect("valid put request becomes deterministic OCI command");

        assert_eq!(command.operation, "PutObject");
        assert_eq!(command.method, "PUT");
        assert_eq!(
            command.endpoint_origin,
            "https://objectstorage.ap-chuncheon-1.oraclecloud.com"
        );
        assert_eq!(
            command.path,
            format!("/n/{NAMESPACE}/b/{BUCKET}/o/workspace%2Freport.pdf")
        );
        assert!(command.body_canonical.contains("object_body_ref=objbody/"));
        assert!(
            command
                .body_canonical
                .contains("data_class=PII_IDENTIFYING")
        );
        assert!(!command.body_canonical.contains("raw_bytes"));
        assert_eq!(
            command.provider_evidence_ref,
            format!("oci-object://{NAMESPACE}/{BUCKET}/{OBJECT_KEY}/storageprov_req_put_001")
        );
    }

    #[test]
    fn get_command_uses_result_reference_without_payload_bytes() {
        let command = adapter()
            .get_command(&get_request())
            .expect("valid get request becomes deterministic OCI command");

        assert_eq!(command.operation, "GetObject");
        assert_eq!(command.method, "GET");
        assert!(command.body_canonical.contains("result_body_ref=objbody/"));
        assert!(!command.body_canonical.contains("raw_bytes"));
    }

    #[test]
    fn object_port_receipts_redact_provider_payloads() {
        let adapter = adapter();
        let put = adapter
            .put_object(put_request())
            .expect("put receipt is generated");
        let get = adapter
            .get_object(get_request())
            .expect("get receipt is generated");

        assert_eq!(put.provider, StorageProviderKind::OciObjectStorage);
        assert_eq!(
            put.provider_request_id,
            "oci-object-1700000000-storageprov_req_put_001"
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
        drifted.provider_bucket_ref = "oci-object://other/bucket".to_string();
        assert!(matches!(
            adapter().put_command(&drifted),
            Err(StorageProviderObjectError::ProviderRejected { .. })
        ));

        let mut bad_bucket = put_request();
        bad_bucket.bucket_id = "oya:cloud:alpha-region:ten_alpha:volume:not-bucket".to_string();
        assert_eq!(
            bad_bucket.validate(),
            Err(StorageProviderObjectError::InvalidRequestShape(
                CloudStorageError::ResourceKindMismatch,
            ))
        );
    }

    #[test]
    fn rejects_invalid_adapter_config() {
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
}
