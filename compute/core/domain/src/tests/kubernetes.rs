use crate::*;

use super::fixtures::*;

#[test]
fn creates_ha_kubernetes_cluster_across_three_azs_with_quota() {
    let cluster = KubernetesCluster::new(k8s_create()).expect("cluster contract is valid");

    assert_eq!(cluster.resource_id.value.kind_label().unwrap(), "k8s");
    assert_eq!(cluster.node_pools.value.len(), 3);
    assert_eq!(
        cluster.control_plane_version.value.value,
        "v1.30.2-oyatie.1"
    );
    assert!(cluster.control_plane_private.value);
    assert_eq!(cluster.schema_version.value, COMPUTE_SCHEMA_VERSION);
}

#[test]
fn rejects_kubernetes_without_ha_spread_or_autoscale_max_node_quota() {
    let ha_error = KubernetesCluster::new(KubernetesClusterCreate {
        node_pools: vec![node_pool(
            "np_a",
            "region-alpha-a",
            "oyatie:cloud:region-alpha:ten_alpha:subnet:prod-a",
        )],
        ..k8s_create()
    })
    .expect_err("HA managed control plane needs three AZs");
    assert_eq!(ha_error, CloudComputeError::KubernetesHaRequiresThreeAzs);

    let quota_error = KubernetesCluster::new(KubernetesClusterCreate {
        quota: ComputeQuotaEnvelope {
            vcpu_limit: 8,
            ..quota()
        },
        ..k8s_create()
    })
    .expect_err("node pool minimum capacity must fit quota");
    assert_eq!(quota_error, CloudComputeError::QuotaExceeded);
    let max_quota_error = KubernetesCluster::new(KubernetesClusterCreate {
        quota: ComputeQuotaEnvelope {
            vcpu_limit: 32,
            ..quota()
        },
        ..k8s_create()
    })
    .expect_err("autoscale maximum capacity must fit quota even when min nodes fit");
    assert_eq!(max_quota_error, CloudComputeError::QuotaExceeded);
}
