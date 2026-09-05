use compute_resource::{InstanceFlavor, K8sFlavor};
use data_boundary_kernel::DataClass;
use network_residency::ResidencyClass;

use crate::*;

use super::fixtures::*;

fn requested_intent() -> KubernetesClusterIntent {
    let create = k8s_create();
    KubernetesClusterIntent {
        resource_id: create.resource_id,
        tenant_id: create.tenant_id,
        region: create.region,
        flavor: create.flavor,
        control_plane_version: create.control_plane_version,
        control_plane_private: create.control_plane_private,
        node_pools: create.node_pools,
        residency: create.residency,
        data_class: create.data_class,
    }
}

#[test]
fn accepts_structural_intent_without_quota_or_creation_time() {
    assert_eq!(
        validate_kubernetes_cluster_intent(requested_intent()),
        Ok(())
    );
}

#[test]
fn rejects_duplicate_pool_identity_without_constructing_cluster() {
    let mut intent = requested_intent();
    intent.node_pools[1].id = intent.node_pools[0].id.clone();

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::DuplicateNodePool)
    );
}

#[test]
fn rejects_invalid_tenant_identity() {
    let mut intent = requested_intent();
    intent.tenant_id = "tenant-alpha".to_string();

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::InvalidTenantId)
    );
}

#[test]
fn rejects_invalid_resource_identity() {
    let mut intent = requested_intent();
    intent.resource_id = "not-a-resource-id".to_string();

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::InvalidResourceId)
    );
}

#[test]
fn rejects_resource_tenant_mismatch() {
    let mut intent = requested_intent();
    intent.resource_id = "oyatie:cloud:region-alpha:ten_beta:k8s:prod".to_string();

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::ResourceTenantMismatch)
    );
}

#[test]
fn rejects_resource_region_mismatch() {
    let mut intent = requested_intent();
    intent.resource_id = "oyatie:cloud:region-beta:ten_alpha:k8s:prod".to_string();

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::ResourceRegionMismatch)
    );
}

#[test]
fn rejects_resource_kind_mismatch() {
    let mut intent = requested_intent();
    intent.resource_id = "oyatie:cloud:region-alpha:ten_alpha:instance:prod".to_string();

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::ResourceKindMismatch)
    );
}

#[test]
fn rejects_invalid_region_identity() {
    let mut intent = requested_intent();
    intent.region = "not a region".to_string();

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::InvalidResourceId)
    );
}

#[test]
fn rejects_invalid_control_plane_version() {
    let mut intent = requested_intent();
    intent.control_plane_version = "1.30".to_string();

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::InvalidControlPlaneVersion)
    );
}

#[test]
fn rejects_high_availability_intent_without_three_availability_zones() {
    let mut intent = requested_intent();
    intent.node_pools.truncate(1);

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::KubernetesHaRequiresThreeAzs)
    );
}

#[test]
fn rejects_invalid_node_pool_bounds() {
    for (min_nodes, max_nodes) in [(0, 5), (5, 4), (1, 1_001)] {
        let mut intent = requested_intent();
        intent.node_pools[0].min_nodes = min_nodes;
        intent.node_pools[0].max_nodes = max_nodes;

        assert_eq!(
            validate_kubernetes_cluster_intent(intent),
            Err(CloudComputeError::InvalidNodePoolShape)
        );
    }
}

#[test]
fn rejects_invalid_node_pool_flavor() {
    let mut intent = requested_intent();
    intent.node_pools[0].flavor.gpu_count = 1;

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::InvalidFlavor)
    );
}

#[test]
fn rejects_empty_node_pool_identity() {
    let mut intent = requested_intent();
    intent.node_pools[0].id.clear();

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::InvalidNodePoolId)
    );
}

#[test]
fn rejects_empty_security_group_set() {
    let mut intent = requested_intent();
    intent.node_pools[0].security_groups.clear();

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::ResourceKindMismatch)
    );
}

#[test]
fn rejects_duplicate_security_group_identity() {
    let mut intent = requested_intent();
    intent.node_pools[0].security_groups = vec!["sg_web".to_string(), "sg_web".to_string()];

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::ResourceKindMismatch)
    );
}

#[test]
fn rejects_residency_that_excludes_requested_region() {
    let mut intent = requested_intent();
    intent.residency = ResidencyClass::StrictHomeRegion;

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::ResidencyRegionMismatch)
    );
}

#[test]
fn rejects_non_public_metadata_classification() {
    let mut intent = requested_intent();
    intent.data_class = DataClass::PiiIdentifying;

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::InvalidDataClass)
    );
}

#[test]
fn rejects_checked_demand_overflow_without_fabricating_quota() {
    let mut intent = requested_intent();
    intent.node_pools[0].flavor = ComputeFlavorSpec {
        class: InstanceFlavor::Gpu,
        vcpu: 1,
        memory_gb: 1,
        gpu_count: u32::MAX / 1_000 + 1,
        local_ssd_gb: 0,
    };
    intent.node_pools[0].max_nodes = 1_000;

    assert_eq!(
        validate_kubernetes_cluster_intent(intent),
        Err(CloudComputeError::QuotaExceeded)
    );
}
