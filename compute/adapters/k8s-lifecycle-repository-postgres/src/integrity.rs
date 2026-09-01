use compute_k8s_api::{
    CloudComputeK8sClusterCreateRequest, CloudComputeK8sClusterRecord,
    CloudComputeK8sLifecycleRepositoryError, validate_cloud_compute_k8s_cluster_record_projection,
};
use sqlx::{Row, postgres::PgRow};

use crate::{SCHEMA_VERSION, error::integrity, operation::decode};

pub(crate) struct StoredCluster {
    pub(crate) desired_spec: CloudComputeK8sClusterCreateRequest,
    pub(crate) cluster: CloudComputeK8sClusterRecord,
}

pub(crate) fn validate_cluster_projection(
    desired_spec: &CloudComputeK8sClusterCreateRequest,
    cluster: &CloudComputeK8sClusterRecord,
) -> Result<(), CloudComputeK8sLifecycleRepositoryError> {
    validate_cloud_compute_k8s_cluster_record_projection(desired_spec, cluster).map_err(integrity)
}

pub(crate) fn decode_stored_cluster(
    row: &PgRow,
    expected_tenant_id: &str,
    expected_resource_id: &str,
) -> Result<StoredCluster, CloudComputeK8sLifecycleRepositoryError> {
    let desired_spec_json: serde_json::Value =
        row.try_get("desired_spec_json").map_err(integrity)?;
    let cluster_json: serde_json::Value = row.try_get("cluster_json").map_err(integrity)?;
    let observed_state: String = row.try_get("observed_state").map_err(integrity)?;
    let desired_state: String = row.try_get("desired_state").map_err(integrity)?;
    let schema_version: i32 = row.try_get("schema_version").map_err(integrity)?;
    let desired_spec: CloudComputeK8sClusterCreateRequest = decode(desired_spec_json)?;
    let cluster: CloudComputeK8sClusterRecord = decode(cluster_json)?;

    validate_cluster_projection(&desired_spec, &cluster)?;
    if schema_version != SCHEMA_VERSION
        || desired_spec.resource_id != expected_resource_id
        || desired_spec.tenant_id != expected_tenant_id
        || cluster.resource_id != expected_resource_id
        || cluster.tenant_id != expected_tenant_id
        || cluster.state != observed_state
        || cluster.desired_state != desired_state
    {
        return Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation);
    }

    Ok(StoredCluster {
        desired_spec,
        cluster,
    })
}
