//! Cloud Region API boundary for region and availability-zone listing.
//!
//! This crate owns authenticated request normalization and public projection for
//! the immutable Cloud region/AZ taxonomy before returning API records.

mod error;
mod model;
mod operations;
mod projection;

pub use model::*;
pub use operations::{
    list_cloud_azs_from_api, list_cloud_regions_from_api, validate_cloud_az_list_request,
    validate_cloud_region_list_request,
};
