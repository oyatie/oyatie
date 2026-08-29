//! Cloud phase-invariant product-surface kernel.
//!
//! This crate owns the ADR-0028 `cloud-surface-kernel` contract: customers
//! bind to one Cloud product surface while the fulfillment substrate moves from
//! rented public-cloud capacity to Oyatie-operated colo to Oyatie-owned DCs.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod cloud_surface;
mod error;
mod fulfillment;
mod ids;
mod sku;
mod surfaces_infra;
mod surfaces_platform;
mod validate;

pub use cloud_surface::{CloudSurface, CloudSurfaceCreate};
pub use error::CloudSurfaceError;
pub use fulfillment::{
    ComputeSkuSurface, ComputeSkuSurfaceCreate, SkuFulfillment, SkuFulfillmentCreate,
};
pub use ids::{CloudSkuId, CloudSurfaceId, ProviderRef};
pub use sku::{
    AcceleratorClass, ColdStartClass, ComputeSku, ComputeSkuKind, FulfillmentPhase,
    FunctionRuntime, InterconnectClass, IsolationLevel, KubeTier, LeaseTerm, NodeClass, PopClass,
    RackClass, VmShape,
};
pub use surfaces_infra::{
    ComputeSurface, ComputeSurfaceCreate, NetworkSurface, NetworkSurfaceCreate, NetworkSurfaceKind,
    StorageSurface, StorageSurfaceCreate, StorageSurfaceKind,
};
pub use surfaces_platform::{
    BillingSurface, BillingSurfaceCreate, FinOpsSurface, FinOpsSurfaceCreate, IamSurface,
    IamSurfaceCreate, ObservabilitySurface, ObservabilitySurfaceCreate, RegionsSurface,
    RegionsSurfaceCreate,
};

/// Schema version stamped onto every emitted surface; part of the contract.
pub const CLOUD_SURFACE_SCHEMA_VERSION: u32 = 1;
pub(crate) const SURFACE_ID_PREFIX: &str = "csurf_";
pub(crate) const SKU_ID_PREFIX: &str = "csku_";
pub(crate) const PROVIDER_REF_PREFIX: &str = "provider/";
pub(crate) const REGION_CODE_PREFIX: &str = "region-";
pub(crate) const MIN_DAY_ONE_AZ_COUNT: u8 = 3;
pub(crate) const MIN_AZ_SEPARATION_KM: u16 = 30;
pub(crate) const MAX_STS_TTL_SECONDS: u32 = 3_600;
