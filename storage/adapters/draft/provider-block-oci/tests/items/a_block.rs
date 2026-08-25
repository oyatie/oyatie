use storage_provider_block_oci_draft::{OciBlockStorageAdapter, OciBlockStorageAdapterConfigError};
use storage_provider_draft::{
    CloudStorageError, DataClass, EncryptionMode, ResidencyClass,
    StorageProviderBlockCreateVolumeRequest, StorageProviderBlockError, StorageProviderBlockPort,
    StorageProviderKind, VolumePerformance, VolumeTier,
};

const COMPARTMENT_REF: &str = "ocid1.compartment.oc1..cloud";
const AVAILABILITY_DOMAIN: &str = "alpha-region-a";
const VOLUME_ID: &str = "oyatie:cloud:alpha-region:ten_alpha:volume:db-primary";

fn block_adapter() -> OciBlockStorageAdapter {
    OciBlockStorageAdapter::new(
        "https://iaas.ap-chuncheon-1.oraclecloud.com",
        COMPARTMENT_REF,
        AVAILABILITY_DOMAIN,
    )
    .unwrap()
    .with_clock(1_700_000_000)
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
    assert!(command.body_canonical.contains("volume_id=oyatie:cloud:"));
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
fn rejects_provider_volume_drift_and_bad_volume_shape() {
    let mut drifted = block_create_request();
    drifted.provider_volume_ref = "oci-block://other/alpha-region-a/volume".to_string();
    assert!(matches!(
        block_adapter().create_volume_command(&drifted),
        Err(StorageProviderBlockError::ProviderRejected { .. })
    ));

    let mut bad_volume = block_create_request();
    bad_volume.volume_id = "oyatie:cloud:alpha-region:ten_alpha:bucket:not-volume".to_string();
    assert_eq!(
        bad_volume.validate(),
        Err(StorageProviderBlockError::InvalidRequestShape(
            CloudStorageError::ResourceKindMismatch,
        ))
    );
}

#[test]
fn rejects_invalid_block_adapter_config() {
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
