//! Cloud Compute managed Kubernetes API boundary for cluster lifecycle.
//!
//! This crate owns request boundary normalization, authorization proof checks,
//! idempotent create and delete semantics, and tenant-safe Kubernetes cluster
//! metadata projection around the Cloud compute kernel. Cluster reconciliation
//! and provider adapters live behind later adapter crates.
//!
//! # Surfaces
//!
//! - [`CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE`] — `cloud.compute.k8s.cluster.create`
//! - [`CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE`] — `cloud.compute.k8s.cluster.delete`

use std::{collections::BTreeMap, future::Future, pin::Pin};

use compute_domain::{
    CloudComputeError, ComputeFlavorSpec, ComputeQuotaEnvelope, ControlPlaneVersion,
    KubernetesCluster, KubernetesClusterCreate, KubernetesClusterDesiredState,
    KubernetesClusterObservation, KubernetesClusterReconcileInput, KubernetesClusterState,
    KubernetesNodePoolCreate, kubernetes_cluster_desired_state_label,
    kubernetes_cluster_state_label, reconcile_kubernetes_cluster,
};
use compute_resource::{InstanceFlavor, K8sFlavor, ResourceId};
use data_boundary_kernel::{DataClass, parse_data_class_label};
use network_residency::{
    ResidencyClass, parse_residency_class_label, residency_class_allows_home_region_label,
};
use serde::{Deserialize, Serialize};
use shared_resource_provider_contract_kernel::OperationState;

pub const CLOUD_COMPUTE_K8S_CLUSTER_RECORD_SCHEMA_VERSION: u32 =
    compute_domain::KUBERNETES_CLUSTER_SCHEMA_VERSION;

include!("create_contract.rs");
include!("create_request.rs");
include!("api_error.rs");
include!("create_flow.rs");
include!("create_projection.rs");
include!("create_result.rs");
include!("delete_contract.rs");
include!("delete_flow.rs");
include!("acceptance_contract.rs");
include!("acceptance_intent.rs");
include!("acceptance_integrity.rs");
include!("acceptance_error.rs");
include!("acceptance_flow.rs");
