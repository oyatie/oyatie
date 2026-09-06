use std::collections::BTreeSet;

use cell_region::{AzCode, CellId, RegionCode};
use compute_resource::{K8sFlavor, ResourceId, ResourceKind};
use data_boundary_kernel::DataClass;
use network_residency::ResidencyClass;

use crate::capacity::ComputeUnits;
use crate::kubernetes::{KubernetesNodePool, KubernetesNodePoolCreate};
use crate::{
    CloudComputeError, ControlPlaneVersion, NodePoolId, public_metadata_class, region_for,
    resource_id_for, resource_ref_for, security_groups, validate_az_region, validate_cell_az,
    validate_tenant_id,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KubernetesClusterIntent {
    pub resource_id: String,                       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                         // data_class: INTERNAL_ONLY
    pub region: String,                            // data_class: PUBLIC
    pub flavor: K8sFlavor,                         // data_class: PUBLIC
    pub control_plane_version: String,             // data_class: PUBLIC
    pub control_plane_private: bool,               // data_class: PUBLIC
    pub node_pools: Vec<KubernetesNodePoolCreate>, // data_class: INTERNAL_ONLY
    pub residency: ResidencyClass,                 // data_class: INTERNAL_ONLY
    pub data_class: DataClass,                     // data_class: PUBLIC
}

pub(crate) struct ValidatedKubernetesClusterIntent {
    pub(crate) resource_id: ResourceId,
    pub(crate) tenant_id: String,
    pub(crate) region: RegionCode,
    pub(crate) flavor: K8sFlavor,
    pub(crate) control_plane_version: ControlPlaneVersion,
    pub(crate) control_plane_private: bool,
    pub(crate) node_pools: Vec<KubernetesNodePool>,
    pub(crate) residency: ResidencyClass,
    pub(crate) requested: ComputeUnits,
}

pub fn validate_kubernetes_cluster_intent(
    input: KubernetesClusterIntent,
) -> Result<(), CloudComputeError> {
    let data_class = input.data_class;
    project_kubernetes_cluster_intent(input)?;
    public_metadata_class(data_class)?;
    Ok(())
}

pub(crate) fn project_kubernetes_cluster_intent(
    input: KubernetesClusterIntent,
) -> Result<ValidatedKubernetesClusterIntent, CloudComputeError> {
    validate_tenant_id(&input.tenant_id)?;
    let region = region_for(&input.region, &input.residency)?;
    let resource_id = resource_id_for(
        &input.resource_id,
        &input.tenant_id,
        &region,
        ResourceKind::KubernetesCluster(input.flavor),
    )?;
    let control_plane_version = ControlPlaneVersion::new(input.control_plane_version)?;
    let node_pools = node_pools(&input.tenant_id, &region, input.node_pools)?;
    validate_kubernetes_shape(input.flavor, &node_pools)?;
    let requested = node_pools
        .iter()
        .try_fold(ComputeUnits::default(), |sum, pool| {
            let count = if pool.autoscaling_enabled {
                pool.max_nodes
            } else {
                pool.min_nodes
            };
            sum.checked_add(pool.flavor.units().checked_mul(count)?)
        })?;

    Ok(ValidatedKubernetesClusterIntent {
        resource_id,
        tenant_id: input.tenant_id,
        region,
        flavor: input.flavor,
        control_plane_version,
        control_plane_private: input.control_plane_private,
        node_pools,
        residency: input.residency,
        requested,
    })
}

fn node_pools(
    tenant_id: &str,
    region: &RegionCode,
    input: Vec<KubernetesNodePoolCreate>,
) -> Result<Vec<KubernetesNodePool>, CloudComputeError> {
    if input.is_empty() {
        return Err(CloudComputeError::InvalidNodePoolShape);
    }
    let mut seen = BTreeSet::new();
    let mut pools = Vec::with_capacity(input.len());
    for pool in input {
        let id = NodePoolId::new(pool.id)?;
        if !seen.insert(id.clone()) {
            return Err(CloudComputeError::DuplicateNodePool);
        }
        let az = AzCode::new(pool.az).map_err(|_| CloudComputeError::InvalidAzCode)?;
        validate_az_region(&az, region)?;
        let cell_id = CellId::new(pool.cell_id).map_err(|_| CloudComputeError::InvalidCellId)?;
        validate_cell_az(&cell_id, &az)?;
        let subnet_id = resource_ref_for(&pool.subnet_id, tenant_id, region, "subnet")?;
        let security_groups = security_groups(pool.security_groups)?;
        let flavor = pool.flavor.validate()?;
        if pool.min_nodes == 0 || pool.max_nodes < pool.min_nodes || pool.max_nodes > 1_000 {
            return Err(CloudComputeError::InvalidNodePoolShape);
        }
        pools.push(KubernetesNodePool {
            id,
            az,
            cell_id,
            subnet_id,
            security_groups,
            flavor,
            min_nodes: pool.min_nodes,
            max_nodes: pool.max_nodes,
            autoscaling_enabled: pool.autoscaling_enabled,
        });
    }
    Ok(pools)
}

fn validate_kubernetes_shape(
    flavor: K8sFlavor,
    node_pools: &[KubernetesNodePool],
) -> Result<(), CloudComputeError> {
    if matches!(flavor, K8sFlavor::HighAvailability) {
        let az_count = node_pools
            .iter()
            .map(|pool| pool.az.clone())
            .collect::<BTreeSet<_>>()
            .len();
        let min_nodes: u32 = node_pools.iter().map(|pool| pool.min_nodes).sum();
        if az_count < 3 || min_nodes < 3 {
            return Err(CloudComputeError::KubernetesHaRequiresThreeAzs);
        }
    }
    Ok(())
}
