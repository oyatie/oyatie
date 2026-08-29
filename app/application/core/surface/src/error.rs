//! Fail-closed validation failures for the cloud surface.

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudSurfaceError {
    InvalidSurfaceId,
    InvalidSkuId,
    InvalidProviderRef,
    InvalidDataClass,
    InvalidFulfillment,
    MissingComputeSkuKind,
    DuplicateComputeSku,
    MissingStorageSurface,
    MissingNetworkSurface,
    InvalidIamSurface,
    InvalidRegionsSurface,
    InvalidBillingSurface,
    InvalidObservabilitySurface,
    InvalidFinOpsSurface,
}
