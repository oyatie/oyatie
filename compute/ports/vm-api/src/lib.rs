//! Cloud Compute VM API boundary for instance creation.
//!
//! This crate owns request boundary normalization, compute-owned authorization
//! verifier checks, idempotent create semantics, and tenant-safe VM metadata
//! the Cloud compute kernel. Hypervisor scheduling and boot orchestration live
//! behind later adapters.

use std::collections::BTreeMap;

use compute_domain::{
    CloudComputeCatalog, CloudComputeError, ComputeFlavorSpec, ComputeQuotaEnvelope, ComputeRepo,
    ImageRefKind, Instance, InstanceCreate, InstanceState,
};
use compute_resource::{InstanceFlavor, ResourceId};
use data_boundary_kernel::{DataClass, parse_data_class_label};
use network_residency::{ResidencyClass, parse_residency_class_label};

include!("authorization.rs");
include!("create_request.rs");
include!("api_error.rs");
include!("create_flow.rs");
include!("create_projection.rs");
include!("domain_error.rs");
