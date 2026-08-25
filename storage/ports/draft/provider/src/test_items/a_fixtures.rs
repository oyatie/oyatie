fn object_put_request() -> StorageProviderObjectPutRequest {
    StorageProviderObjectPutRequest {
        request_id: "storageprov_req_put_001".to_string(),
        provider_bucket_ref: "s3://tenant-assets".to_string(),
        bucket_id: "oyatie:cloud:region-alpha1:ten_alpha:bucket:tenant-assets".to_string(),
        tenant_id: "ten_alpha".to_string(),
        object_key: "workspace/report.pdf".to_string(),
        object_body_ref: "objbody/ten_alpha/workspace/report".to_string(),
        size_bytes: 42,
        etag: "0123456789abcdef0123456789abcdef".to_string(),
        data_class: DataClass::PiiIdentifying,
        kms_key: "kms/region-alpha1/ten_alpha/object-key".to_string(),
        ciphertext_ref: "ct/ten_alpha/object/report".to_string(),
        actor: "sp_storage".to_string(),
        idempotency_key: "idem-storage-object-put".to_string(),
        requested_at_epoch_seconds: 1_700_000_010,
    }
}

fn object_get_request() -> StorageProviderObjectGetRequest {
    StorageProviderObjectGetRequest {
        request_id: "storageprov_req_get_001".to_string(),
        provider_bucket_ref: "s3://tenant-assets".to_string(),
        bucket_id: "oyatie:cloud:region-alpha1:ten_alpha:bucket:tenant-assets".to_string(),
        tenant_id: "ten_alpha".to_string(),
        object_key: "workspace/report.pdf".to_string(),
        result_body_ref: "objbody/ten_alpha/workspace/report-read".to_string(),
        actor: "sp_storage".to_string(),
        requested_at_epoch_seconds: 1_700_000_020,
    }
}

fn block_create_request() -> StorageProviderBlockCreateVolumeRequest {
    StorageProviderBlockCreateVolumeRequest {
        request_id: "storageprov_req_block_create_001".to_string(),
        provider_volume_ref: "oci-block://compartment/region-alpha1-a/db-primary".to_string(),
        volume_id: "oyatie:cloud:region-alpha1:ten_alpha:volume:db-primary".to_string(),
        tenant_id: "ten_alpha".to_string(),
        name: "db-primary".to_string(),
        region: "region-alpha1".to_string(),
        az: "region-alpha1-a".to_string(),
        cell_id: "cell-region-alpha1-a-001".to_string(),
        residency: ResidencyClass::Global,
        tier: VolumeTier::ProvisionedIopsSsd,
        size_gib: 512,
        performance: VolumePerformance {
            iops: 12_000,
            throughput_mbps: 750,
        },
        encryption: EncryptionMode::Byok,
        kms_key: Some("byok/region-alpha1/ten_alpha/db-key".to_string()),
        data_class: DataClass::PiiIdentifying,
        actor: "sp_storage".to_string(),
        idempotency_key: "idem-storage-block-create".to_string(),
        requested_at_epoch_seconds: 1_700_000_010,
    }
}
