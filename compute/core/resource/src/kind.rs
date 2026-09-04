#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InstanceFlavor {
    GeneralPurpose,
    ComputeOptimized,
    MemoryOptimized,
    Gpu,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum K8sFlavor {
    Standard,
    HighAvailability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FunctionRuntime {
    Rust,
    TypeScript,
    Python,
    Wasm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BareMetalFlavor {
    GeneralPurpose,
    StorageOptimized,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GpuFlavor {
    Training,
    Inference,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BucketTier {
    Standard,
    InfrequentAccess,
    Archive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum VolumeTier {
    GeneralPurposeSsd,
    ProvisionedIopsSsd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FilesystemTier {
    Standard,
    ThroughputOptimized,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LbProtocol {
    L4,
    L7,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DatabaseEngine {
    Postgres,
    Citus,
    PgVector,
    Valkey,
    Kafka,
    ClickHouse,
    Cassandra,
    Iceberg,
    Milvus,
    Temporal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum QueueEngine {
    Kafka,
    Nats,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ImageKind {
    MachineImage,
    ContainerImage,
    FunctionBundle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ResourceKind {
    ComputeInstance(InstanceFlavor),
    KubernetesCluster(K8sFlavor),
    Function(FunctionRuntime),
    BareMetal(BareMetalFlavor),
    GpuFleet(GpuFlavor),
    Bucket(BucketTier),
    Volume(VolumeTier),
    Filesystem(FilesystemTier),
    ArchiveVault,
    Vpc,
    Subnet,
    LoadBalancer(LbProtocol),
    DnsZone,
    CdnDistribution,
    DirectInterconnect,
    DdosProtection,
    Database(DatabaseEngine),
    QueueOrStream(QueueEngine),
    SearchIndex,
    KmsKey,
    SecretBundle,
    Image(ImageKind),
}

impl ResourceKind {
    pub const fn type_label(self) -> &'static str {
        match self {
            Self::ComputeInstance(_) => "instance",
            Self::KubernetesCluster(_) => "k8s",
            Self::Function(_) => "function",
            Self::BareMetal(_) => "bare-metal",
            Self::GpuFleet(_) => "gpu-fleet",
            Self::Bucket(_) => "bucket",
            Self::Volume(_) => "volume",
            Self::Filesystem(_) => "filesystem",
            Self::ArchiveVault => "archive-vault",
            Self::Vpc => "vpc",
            Self::Subnet => "subnet",
            Self::LoadBalancer(LbProtocol::L4) => "lb-v4",
            Self::LoadBalancer(LbProtocol::L7) => "lb-v7",
            Self::DnsZone => "dns-zone",
            Self::CdnDistribution => "cdn-distribution",
            Self::DirectInterconnect => "direct-interconnect",
            Self::DdosProtection => "ddos-protection",
            Self::Database(_) => "database",
            Self::QueueOrStream(_) => "queue-stream",
            Self::SearchIndex => "search-index",
            Self::KmsKey => "kms-key",
            Self::SecretBundle => "secret-bundle",
            Self::Image(_) => "image",
        }
    }

    pub const fn requires_az(self) -> bool {
        matches!(
            self,
            Self::ComputeInstance(_)
                | Self::KubernetesCluster(_)
                | Self::Function(_)
                | Self::BareMetal(_)
                | Self::GpuFleet(_)
                | Self::Volume(_)
                | Self::Filesystem(_)
                | Self::Subnet
                | Self::LoadBalancer(_)
                | Self::Database(_)
                | Self::QueueOrStream(_)
                | Self::SearchIndex
                | Self::Image(_)
        )
    }
}
