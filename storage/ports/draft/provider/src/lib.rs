//! Unagreed object and block backend-provider contract.
//!
//! This owner-local draft isolates removable S3/OCI backend semantics from the
//! storage core. It is not the sold S3 facade and must not be consumed by other
//! owners. `storage-domain` re-exports these types during the P0 compatibility
//! window.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use cell_region::{AzCode, CellId, RegionCode};
pub use compute_resource::VolumeTier;
use compute_resource::{CloudResourceError, PrincipalId, ResourceId, ResourceKind};
pub use data_boundary_kernel::DataClass;
use data_boundary_kernel::PrivacyDataClass;
pub use network_residency::ResidencyClass;
use network_residency::residency_class_allows_home_region_label;
use secrets_kms_domain::{CiphertextRef, KmsKeyId, KmsKeyOrigin};

include!(concat!(env!("OUT_DIR"), "/provider.generated.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    include!(concat!(env!("OUT_DIR"), "/provider_tests.generated.rs"));
}
