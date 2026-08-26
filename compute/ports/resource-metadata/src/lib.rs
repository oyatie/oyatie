//! Agreed cross-owner surface for cloud-resource identity and kind metadata.
//!
//! Consumers receive Compute's established exact value identities, Cell's
//! agreed location identity, typed validation errors, and complete resource-kind
//! vocabulary without importing Compute's internal aggregate. This port
//! intentionally excludes resource state, registry/repository behavior, policy
//! attachments, tags, and metering. The legacy resource core remains the
//! defining crate until its large aggregate is decomposed in a dedicated
//! Compute structural lane.

#![forbid(unsafe_code)]

pub use cell_location::RegionCode;
pub use compute_resource::{
    BareMetalFlavor, BucketTier, CloudResourceError, DatabaseEngine, FilesystemTier,
    FunctionRuntime, GpuFlavor, ImageKind, InstanceFlavor, K8sFlavor, LbProtocol, PrincipalId,
    QueueEngine, ResourceId, ResourceKind, VolumeTier,
};
