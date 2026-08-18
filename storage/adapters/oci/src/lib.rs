//! OCI storage adapter boundary for Cloud Storage.
//!
//! This crate keeps OCI namespace, bucket, compartment, volume path, and evidence
//! refs outside provider-neutral Cloud Storage domain/API crates while
//! implementing shared storage provider port contracts. It builds deterministic
//! request shapes; credentialed live smoke remains a separate promotion gate.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use storage_domain::{
    EncryptionMode, StorageProviderBlockCreateVolumeRequest, StorageProviderBlockError,
    StorageProviderBlockPort, StorageProviderBlockReceipt, StorageProviderKind,
    StorageProviderObjectError, StorageProviderObjectGetRequest, StorageProviderObjectPort,
    StorageProviderObjectPutRequest, StorageProviderObjectReceipt, VolumeTier,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OciObjectStorageAdapterConfigError {
    InvalidEndpoint,
    InvalidNamespace,
    InvalidBucketName,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OciBlockStorageAdapterConfigError {
    InvalidEndpoint,
    InvalidCompartmentRef,
    InvalidAvailabilityDomain,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciBlockStorageAdapter {
    endpoint_origin: String,     // data_class: INTERNAL_ONLY
    compartment_ref: String,     // data_class: INTERNAL_ONLY
    availability_domain: String, // data_class: INTERNAL_ONLY
    clock_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciBlockStorageCommand {
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

impl OciBlockStorageAdapter {
    pub fn new(
        endpoint_origin: impl Into<String>,
        compartment_ref: impl Into<String>,
        availability_domain: impl Into<String>,
    ) -> Result<Self, OciBlockStorageAdapterConfigError> {
        let endpoint_origin = endpoint_origin.into();
        let compartment_ref = compartment_ref.into();
        let availability_domain = availability_domain.into();
        validate_block_endpoint(&endpoint_origin)?;
        validate_block_segment(
            &compartment_ref,
            OciBlockStorageAdapterConfigError::InvalidCompartmentRef,
        )?;
        validate_block_segment(
            &availability_domain,
            OciBlockStorageAdapterConfigError::InvalidAvailabilityDomain,
        )?;
        Ok(Self {
            endpoint_origin,
            compartment_ref,
            availability_domain,
            clock_epoch_seconds: 0,
        })
    }

    pub fn with_clock(mut self, clock_epoch_seconds: u64) -> Self {
        self.clock_epoch_seconds = clock_epoch_seconds;
        self
    }

    pub fn provider_volume_ref(&self, volume_id: &str) -> String {
        format!(
            "oci-block://{}/{}/{}",
            self.compartment_ref, self.availability_domain, volume_id
        )
    }

    pub fn create_volume_command(
        &self,
        request: &StorageProviderBlockCreateVolumeRequest,
    ) -> Result<OciBlockStorageCommand, StorageProviderBlockError> {
        request.validate()?;
        self.ensure_provider_volume(&request.provider_volume_ref, &request.volume_id)?;
        let size_gib = request.size_gib.to_string();
        let iops = request.performance.iops.to_string();
        let throughput_mbps = request.performance.throughput_mbps.to_string();
        let requested_at = request.requested_at_epoch_seconds.to_string();
        let kms_key = request.kms_key.as_deref().unwrap_or("");
        Ok(OciBlockStorageCommand {
            operation: "CreateVolume",
            method: "POST",
            endpoint_origin: self.endpoint_origin.clone(),
            path: "/20160918/volumes".to_string(),
            body_canonical: canonical_body(&[
                ("compartment_ref", self.compartment_ref.as_str()),
                ("availability_domain", self.availability_domain.as_str()),
                ("volume_id", request.volume_id.as_str()),
                ("tenant_id", request.tenant_id.as_str()),
                ("name", request.name.as_str()),
                ("region", request.region.as_str()),
                ("az", request.az.as_str()),
                ("cell_id", request.cell_id.as_str()),
                ("residency", request.residency.label().unwrap_or("per_pack")),
                ("tier", volume_tier_label(request.tier)),
                ("size_gib", size_gib.as_str()),
                ("iops", iops.as_str()),
                ("throughput_mbps", throughput_mbps.as_str()),
                ("encryption", encryption_label(request.encryption)),
                ("kms_key", kms_key),
                ("data_class", request.data_class.label()),
                ("actor", request.actor.as_str()),
                ("idempotency_key", request.idempotency_key.as_str()),
                ("requested_at_epoch_seconds", requested_at.as_str()),
            ]),
            provider_evidence_ref: format!(
                "oci-block://{}/{}/{}/{}",
                self.compartment_ref,
                self.availability_domain,
                request.volume_id,
                request.request_id
            ),
        })
    }

    fn ensure_provider_volume(
        &self,
        provider_volume_ref: &str,
        volume_id: &str,
    ) -> Result<(), StorageProviderBlockError> {
        let expected = self.provider_volume_ref(volume_id);
        if provider_volume_ref == expected {
            Ok(())
        } else {
            Err(StorageProviderBlockError::ProviderRejected {
                provider: StorageProviderKind::OciBlockStorage,
                reason: "provider_volume_ref does not match configured OCI Block Volume target"
                    .to_string(),
            })
        }
    }

    fn provider_request_id(&self, request_id: &str) -> String {
        format!("oci-block-{}-{request_id}", self.clock_epoch_seconds)
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

impl StorageProviderBlockPort for OciBlockStorageAdapter {
    fn provider_kind(&self) -> StorageProviderKind {
        StorageProviderKind::OciBlockStorage
    }

    fn create_volume(
        &self,
        input: StorageProviderBlockCreateVolumeRequest,
    ) -> Result<StorageProviderBlockReceipt, StorageProviderBlockError> {
        let command = self.create_volume_command(&input)?;
        StorageProviderBlockReceipt::create_volume(
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

fn validate_block_endpoint(value: &str) -> Result<(), OciBlockStorageAdapterConfigError> {
    if value.starts_with("https://") && no_space_or_control(value) {
        Ok(())
    } else {
        Err(OciBlockStorageAdapterConfigError::InvalidEndpoint)
    }
}

fn validate_block_segment(
    value: &str,
    error: OciBlockStorageAdapterConfigError,
) -> Result<(), OciBlockStorageAdapterConfigError> {
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
    object_key
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect::<String>()
}

fn canonical_body(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("|")
}

const fn volume_tier_label(tier: VolumeTier) -> &'static str {
    match tier {
        VolumeTier::GeneralPurposeSsd => "general_purpose_ssd",
        VolumeTier::ProvisionedIopsSsd => "provisioned_iops_ssd",
    }
}

const fn encryption_label(mode: EncryptionMode) -> &'static str {
    match mode {
        EncryptionMode::Sse => "sse",
        EncryptionMode::SseKms => "sse_kms",
        EncryptionMode::Byok => "byok",
        EncryptionMode::Hyok => "hyok",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_data_boundary_kernel::DataClass;
    use storage_domain::{
        CloudStorageError, ResidencyClass, StorageProviderBlockCreateVolumeRequest,
        VolumePerformance,
    };

    const NAMESPACE: &str = "axdotp9iv3ua";
    const BUCKET: &str = "oyatie-audit-cold-backup";
    const BUCKET_ID: &str = "oya:cloud:alpha-region:ten_alpha:bucket:tenant-assets";
    const OBJECT_KEY: &str = "workspace/report.pdf";
    const COMPARTMENT_REF: &str = "ocid1.compartment.oc1..cloud";
    const AVAILABILITY_DOMAIN: &str = "alpha-region-a";
    const VOLUME_ID: &str = "oya:cloud:alpha-region:ten_alpha:volume:db-primary";

    fn adapter() -> OciObjectStorageAdapter {
        OciObjectStorageAdapter::new(
            "https://objectstorage.ap-chuncheon-1.oraclecloud.com",
            NAMESPACE,
            BUCKET,
        )
        .unwrap()
        .with_clock(1_700_000_000)
    }

    fn block_adapter() -> OciBlockStorageAdapter {
        OciBlockStorageAdapter::new(
            "https://iaas.ap-chuncheon-1.oraclecloud.com",
            COMPARTMENT_REF,
            AVAILABILITY_DOMAIN,
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

    fn block_create_request() -> StorageProviderBlockCreateVolumeRequest {
        StorageProviderBlockCreateVolumeRequest {
            request_id: "storageprov_req_block_create_001".to_string(),
            provider_volume_ref: format!(
                "oci-block://{COMPARTMENT_REF}/{AVAILABILITY_DOMAIN}/{VOLUME_ID}"
            ),
            volume_id: VOLUME_ID.to_string(),
            tenant_id: "ten_alpha".to_string(),
            name: "db-primary".to_string(),
            region: "alpha-region".to_string(),
            az: "alpha-region-a".to_string(),
            cell_id: "cell-alpha-region-a-001".to_string(),
            residency: ResidencyClass::Global,
            tier: VolumeTier::ProvisionedIopsSsd,
            size_gib: 512,
            performance: VolumePerformance {
                iops: 12_000,
                throughput_mbps: 750,
            },
            encryption: EncryptionMode::Byok,
            kms_key: Some("byok/alpha-region/ten_alpha/db-key".to_string()),
            data_class: DataClass::PiiIdentifying,
            actor: "sp_storage".to_string(),
            idempotency_key: "idem-storage-block-create".to_string(),
            requested_at_epoch_seconds: 1_700_000_010,
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
    fn object_path_percent_encodes_provider_url_segments() {
        let mut request = get_request();
        request.object_key = "workspace/final report #1.pdf".to_string();

        let command = adapter()
            .get_command(&request)
            .expect("object key with spaces remains a valid domain key");

        assert_eq!(
            command.path,
            format!("/n/{NAMESPACE}/b/{BUCKET}/o/workspace%2Ffinal%20report%20%231.pdf")
        );
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
    fn create_volume_command_uses_oci_block_path_and_reference_only_body() {
        let command = block_adapter()
            .create_volume_command(&block_create_request())
            .expect("valid block request becomes deterministic OCI command");

        assert_eq!(command.operation, "CreateVolume");
        assert_eq!(command.method, "POST");
        assert_eq!(
            command.endpoint_origin,
            "https://iaas.ap-chuncheon-1.oraclecloud.com"
        );
        assert_eq!(command.path, "/20160918/volumes");
        assert!(command.body_canonical.contains("compartment_ref=ocid1."));
        assert!(
            command
                .body_canonical
                .contains("availability_domain=alpha-region-a")
        );
        assert!(command.body_canonical.contains("volume_id=oya:cloud:"));
        assert!(command.body_canonical.contains("tier=provisioned_iops_ssd"));
        assert!(command.body_canonical.contains("encryption=byok"));
        assert!(
            command
                .body_canonical
                .contains("data_class=PII_IDENTIFYING")
        );
        assert!(!command.body_canonical.contains("private_key"));
        assert_eq!(
            command.provider_evidence_ref,
            format!(
                "oci-block://{COMPARTMENT_REF}/{AVAILABILITY_DOMAIN}/{VOLUME_ID}/storageprov_req_block_create_001"
            )
        );
    }

    #[test]
    fn block_port_receipts_preserve_refs_without_provider_credentials() {
        let receipt = block_adapter()
            .create_volume(block_create_request())
            .expect("block receipt is generated");

        assert_eq!(receipt.provider, StorageProviderKind::OciBlockStorage);
        assert_eq!(
            receipt.provider_request_id,
            "oci-block-1700000000-storageprov_req_block_create_001"
        );
        assert_eq!(
            receipt.provider_volume_ref,
            format!("oci-block://{COMPARTMENT_REF}/{AVAILABILITY_DOMAIN}/{VOLUME_ID}")
        );
        assert_eq!(receipt.volume_id, VOLUME_ID);
        assert_eq!(
            receipt.kms_key.as_deref(),
            Some("byok/alpha-region/ten_alpha/db-key")
        );
        assert_eq!(receipt.actor, "sp_storage");
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
    fn rejects_provider_volume_drift_and_bad_volume_shape() {
        let mut drifted = block_create_request();
        drifted.provider_volume_ref = "oci-block://other/alpha-region-a/volume".to_string();
        assert!(matches!(
            block_adapter().create_volume_command(&drifted),
            Err(StorageProviderBlockError::ProviderRejected { .. })
        ));

        let mut bad_volume = block_create_request();
        bad_volume.volume_id = "oya:cloud:alpha-region:ten_alpha:bucket:not-volume".to_string();
        assert_eq!(
            bad_volume.validate(),
            Err(StorageProviderBlockError::InvalidRequestShape(
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
        assert_eq!(
            OciBlockStorageAdapter::new("http://iaas", COMPARTMENT_REF, AVAILABILITY_DOMAIN,),
            Err(OciBlockStorageAdapterConfigError::InvalidEndpoint)
        );
        assert_eq!(
            OciBlockStorageAdapter::new(
                "https://iaas.ap-chuncheon-1.oraclecloud.com",
                "bad compartment",
                AVAILABILITY_DOMAIN,
            ),
            Err(OciBlockStorageAdapterConfigError::InvalidCompartmentRef)
        );
    }
}
