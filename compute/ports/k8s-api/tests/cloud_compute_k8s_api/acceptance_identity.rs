use compute_k8s_api::{
    CloudComputeK8sAcceptanceContract, CloudComputeK8sClusterCreateIntent,
    cloud_compute_k8s_create_intent_fingerprint, validate_cloud_compute_k8s_create_intent,
};

use super::acceptance_test_repository::pending_intent;

#[test]
fn every_stable_intent_and_pool_field_participates_in_identity() {
    let base = pending_intent();
    let fingerprint = cloud_compute_k8s_create_intent_fingerprint(&base).unwrap();
    macro_rules! distinct {
        ($mutation:expr) => {{
            let mut changed = base.clone();
            $mutation(&mut changed as &mut CloudComputeK8sClusterCreateIntent);
            assert_ne!(
                cloud_compute_k8s_create_intent_fingerprint(&changed),
                Ok(fingerprint.clone())
            );
        }};
    }
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.resource_id =
        "oyatie:cloud:region-home:ten_alpha:k8s:other".into());
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.tenant_id = "ten_other".into());
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.region = "region-other".into());
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.flavor = "standard".into());
    distinct!(
        |v: &mut CloudComputeK8sClusterCreateIntent| v.control_plane_version =
            "v1.30.3-oyatie.1".into()
    );
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.control_plane_private = false);
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.residency = "per_pack".into());
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.data_class = "INTERNAL_ONLY".into());
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.node_pools[0].id = "np_z".into());
    distinct!(
        |v: &mut CloudComputeK8sClusterCreateIntent| v.node_pools[0].az = "region-home-z".into()
    );
    distinct!(
        |v: &mut CloudComputeK8sClusterCreateIntent| v.node_pools[0].cell_id =
            "cell-region-home-a-002".into()
    );
    distinct!(
        |v: &mut CloudComputeK8sClusterCreateIntent| v.node_pools[0].subnet_id =
            "oyatie:cloud:region-home:ten_alpha:subnet:other".into()
    );
    distinct!(
        |v: &mut CloudComputeK8sClusterCreateIntent| v.node_pools[0].security_groups[0] =
            "sg_other".into()
    );
    distinct!(
        |v: &mut CloudComputeK8sClusterCreateIntent| v.node_pools[0].flavor.class =
            "compute_optimized".into()
    );
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.node_pools[0].flavor.vcpu = 8);
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.node_pools[0].flavor.memory_gb = 32);
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| {
        v.node_pools[0].flavor.class = "gpu".into();
        v.node_pools[0].flavor.gpu_count = 1;
    });
    distinct!(
        |v: &mut CloudComputeK8sClusterCreateIntent| v.node_pools[0].flavor.local_ssd_gb = 200
    );
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.node_pools[0].min_nodes = 2);
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.node_pools[0].max_nodes = 6);
    distinct!(
        |v: &mut CloudComputeK8sClusterCreateIntent| v.node_pools[0].autoscaling_enabled = false
    );
}

#[test]
fn canonical_identity_ignores_pool_and_group_order_but_rejects_ambiguity() {
    let base = pending_intent();
    let mut reordered = base.clone();
    reordered.node_pools.reverse();
    for pool in &mut reordered.node_pools {
        pool.security_groups.reverse();
    }
    assert_eq!(
        cloud_compute_k8s_create_intent_fingerprint(&base).unwrap(),
        cloud_compute_k8s_create_intent_fingerprint(&reordered).unwrap(),
    );

    let mut duplicate_pool = base.clone();
    duplicate_pool
        .node_pools
        .push(duplicate_pool.node_pools[0].clone());
    assert!(validate_cloud_compute_k8s_create_intent(&duplicate_pool).is_err());
    let mut duplicate_group = base;
    duplicate_group.node_pools[0].security_groups[1] =
        duplicate_group.node_pools[0].security_groups[0].clone();
    assert!(validate_cloud_compute_k8s_create_intent(&duplicate_group).is_err());
}

#[test]
fn persisted_intent_has_no_quota_timestamp_or_authorization_claims() {
    let encoded = serde_json::to_value(pending_intent()).unwrap();
    let object = encoded.as_object().unwrap();
    for absent in [
        "quota",
        "created_at_epoch_seconds",
        "authorization",
        "claims",
    ] {
        assert!(!object.contains_key(absent), "unexpected field {absent}");
    }
    assert!(
        serde_json::from_str::<CloudComputeK8sAcceptanceContract>("\"unknown_contract\"").is_err()
    );
}
