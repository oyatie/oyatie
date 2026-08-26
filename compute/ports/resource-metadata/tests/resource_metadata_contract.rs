use compute_resource_metadata::{
    BareMetalFlavor, BucketTier, CloudResourceError, DatabaseEngine, FilesystemTier,
    FunctionRuntime, GpuFlavor, ImageKind, InstanceFlavor, K8sFlavor, LbProtocol, PrincipalId,
    QueueEngine, RegionCode, ResourceId, ResourceKind, VolumeTier,
};

#[test]
fn resource_identity_preserves_exact_parsing_and_errors() {
    let resource_id = ResourceId::new("oyatie:cloud:region-alpha1:ten_alpha:bucket:tenant-assets")
        .expect("canonical resource id should construct");

    assert_eq!(resource_id.tenant_id(), Ok("ten_alpha".to_string()));
    let region: RegionCode = resource_id.region().expect("canonical region should parse");
    assert_eq!(region.value, "region-alpha1");
    assert_eq!(resource_id.kind_label(), Ok("bucket".to_string()));
    assert_eq!(resource_id.resource_name(), Ok("tenant-assets".to_string()));
    assert_eq!(
        ResourceId::new("cloud:region-alpha1:ten_alpha:bucket:tenant-assets"),
        Err(CloudResourceError::InvalidResourceId)
    );
}

#[test]
fn principal_identity_preserves_exact_validation_errors() {
    assert!(PrincipalId::new("usr_operator").is_ok());
    assert!(PrincipalId::new("sp_storage-gateway").is_ok());
    assert_eq!(
        PrincipalId::new("operator"),
        Err(CloudResourceError::InvalidPrincipalId)
    );
}

#[test]
fn storage_resource_kind_and_tier_vocabulary_remains_stable() {
    for tier in [
        BucketTier::Standard,
        BucketTier::InfrequentAccess,
        BucketTier::Archive,
    ] {
        let kind = ResourceKind::Bucket(tier);
        assert_eq!(kind.type_label(), "bucket");
        assert!(!kind.requires_az());
    }
    for tier in [
        VolumeTier::GeneralPurposeSsd,
        VolumeTier::ProvisionedIopsSsd,
    ] {
        let kind = ResourceKind::Volume(tier);
        assert_eq!(kind.type_label(), "volume");
        assert!(kind.requires_az());
    }
    for tier in [
        FilesystemTier::Standard,
        FilesystemTier::ThroughputOptimized,
    ] {
        let kind = ResourceKind::Filesystem(tier);
        assert_eq!(kind.type_label(), "filesystem");
        assert!(kind.requires_az());
    }
    assert_eq!(ResourceKind::ArchiveVault.type_label(), "archive-vault");
    assert!(!ResourceKind::ArchiveVault.requires_az());
}

#[test]
fn every_resource_kind_payload_type_is_nameable_through_the_port() {
    let cases = [
        (
            ResourceKind::ComputeInstance(InstanceFlavor::GeneralPurpose),
            "instance",
        ),
        (ResourceKind::KubernetesCluster(K8sFlavor::Standard), "k8s"),
        (ResourceKind::Function(FunctionRuntime::Rust), "function"),
        (
            ResourceKind::BareMetal(BareMetalFlavor::GeneralPurpose),
            "bare-metal",
        ),
        (ResourceKind::GpuFleet(GpuFlavor::Inference), "gpu-fleet"),
        (ResourceKind::Bucket(BucketTier::Standard), "bucket"),
        (
            ResourceKind::Volume(VolumeTier::GeneralPurposeSsd),
            "volume",
        ),
        (
            ResourceKind::Filesystem(FilesystemTier::Standard),
            "filesystem",
        ),
        (ResourceKind::ArchiveVault, "archive-vault"),
        (ResourceKind::Vpc, "vpc"),
        (ResourceKind::Subnet, "subnet"),
        (ResourceKind::LoadBalancer(LbProtocol::L4), "lb-v4"),
        (ResourceKind::DnsZone, "dns-zone"),
        (ResourceKind::CdnDistribution, "cdn-distribution"),
        (ResourceKind::DirectInterconnect, "direct-interconnect"),
        (ResourceKind::DdosProtection, "ddos-protection"),
        (ResourceKind::Database(DatabaseEngine::Postgres), "database"),
        (
            ResourceKind::QueueOrStream(QueueEngine::Nats),
            "queue-stream",
        ),
        (ResourceKind::SearchIndex, "search-index"),
        (ResourceKind::KmsKey, "kms-key"),
        (ResourceKind::SecretBundle, "secret-bundle"),
        (ResourceKind::Image(ImageKind::ContainerImage), "image"),
    ];

    for (kind, expected_label) in cases {
        assert_eq!(kind.type_label(), expected_label);
    }
}
