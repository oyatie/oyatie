//! Cloud phase-invariant product-surface kernel (ADR-0028).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const CLOUD_SURFACE_SCHEMA_VERSION: u32 = 1;
const SURFACE_ID_PREFIX: &str = "csurf_";
const SKU_ID_PREFIX: &str = "csku_";
const PROVIDER_REF_PREFIX: &str = "provider/";
const REGION_CODE_PREFIX: &str = "region-";
const MIN_DAY_ONE_AZ_COUNT: u8 = 3;
const MIN_AZ_SEPARATION_KM: u16 = 30;
const MAX_STS_TTL_SECONDS: u32 = 3_600;

mod ids;
pub use ids::*;
mod sku;
pub use sku::*;
mod model;
pub use model::*;
mod impls;
#[cfg(test)]
mod tests;
mod validate;
