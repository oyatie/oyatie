//! Cloud Compute Functions API boundary for invocation receipts.
//!
//! This crate owns request boundary normalization, authorization proof checks,
//! idempotent invoke semantics, and tenant-safe function invocation projection
//! around the Cloud compute kernel.

use std::collections::BTreeMap;

use compute_domain::{
    CloudComputeCatalog, CloudComputeError, ComputeRepo, FunctionInvocationReceipt,
    FunctionInvocationRequest,
};
use compute_resource::ResourceId;
use data_boundary_kernel::{DataClass, parse_data_class_label};

include!("authorization.rs");
include!("invocation_contract.rs");
include!("api_error.rs");
include!("request_flow.rs");
include!("idempotency.rs");
include!("domain_error.rs");
