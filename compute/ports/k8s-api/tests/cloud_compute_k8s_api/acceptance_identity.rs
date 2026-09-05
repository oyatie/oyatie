use compute_domain::CloudComputeError;
use compute_k8s_api::{
    CloudComputeK8sAcceptanceContract, CloudComputeK8sApiError, CloudComputeK8sClusterCreateIntent,
    cloud_compute_k8s_create_intent_fingerprint, validate_cloud_compute_k8s_create_intent,
};

use super::acceptance_test_repository::pending_intent;

fn assert_valid_distinct(
    base: &CloudComputeK8sClusterCreateIntent,
    changed: &CloudComputeK8sClusterCreateIntent,
) {
    let base = cloud_compute_k8s_create_intent_fingerprint(base).unwrap();
    let changed = cloud_compute_k8s_create_intent_fingerprint(changed).unwrap();
    assert_ne!(changed, base);
}

#[test]
fn every_independently_variable_intent_field_participates_in_identity() {
    let base = pending_intent();
    macro_rules! distinct {
        ($mutation:expr) => {{
            let mut changed = base.clone();
            $mutation(&mut changed as &mut CloudComputeK8sClusterCreateIntent);
            assert_valid_distinct(&base, &changed);
        }};
    }
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.resource_id =
        "oyatie:cloud:region-home:ten_alpha:k8s:other".into());
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| {
        v.tenant_id = "ten_other".into();
        v.resource_id = "oyatie:cloud:region-home:ten_other:k8s:prod".into();
        for pool in &mut v.node_pools {
            pool.subnet_id = pool.subnet_id.replace("ten_alpha", "ten_other");
        }
    });
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| {
        v.region = "region-home-2".into();
        v.resource_id = v.resource_id.replace("region-home", "region-home-2");
        for pool in &mut v.node_pools {
            pool.az = pool.az.replace("region-home", "region-home-2");
            pool.cell_id = pool.cell_id.replace("region-home", "region-home-2");
            pool.subnet_id = pool.subnet_id.replace("region-home", "region-home-2");
        }
    });
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.flavor = "standard".into());
    distinct!(
        |v: &mut CloudComputeK8sClusterCreateIntent| v.control_plane_version =
            "v1.30.3-oyatie.1".into()
    );
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.control_plane_private = false);
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.residency = "global".into());
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| v.node_pools[0].id = "np_z".into());
    distinct!(|v: &mut CloudComputeK8sClusterCreateIntent| {
        v.node_pools[0].az = "region-home-z".into();
        v.node_pools[0].cell_id = "cell-region-home-z-001".into();
    });
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
fn gpu_count_alone_participates_in_identity_for_valid_gpu_intents() {
    let mut one_gpu = pending_intent();
    one_gpu.node_pools[0].flavor.class = "gpu".into();
    one_gpu.node_pools[0].flavor.gpu_count = 1;
    let mut two_gpus = one_gpu.clone();
    two_gpus.node_pools[0].flavor.gpu_count = 2;
    assert_valid_distinct(&one_gpu, &two_gpus);
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
fn canonical_fingerprint_matches_the_hand_checked_fixed_field_representation() {
    let fingerprint = cloud_compute_k8s_create_intent_fingerprint(&pending_intent()).unwrap();
    let encoded: serde_json::Value = serde_json::from_str(&fingerprint).unwrap();
    let expected = serde_json::json!(["pending_intent", {
        "resource_id": "oyatie:cloud:region-home:ten_alpha:k8s:prod",
        "tenant_id": "ten_alpha",
        "region": "region-home",
        "flavor": "high_availability",
        "control_plane_version": "v1.30.2-oyatie.1",
        "control_plane_private": true,
        "node_pools": [
            {"id":"np_a","az":"region-home-a","cell_id":"cell-region-home-a-001","subnet_id":"oyatie:cloud:region-home:ten_alpha:subnet:prod-a","security_groups":["sg_np_a_app","sg_np_a_web"],"flavor":{"class":"general_purpose","vcpu":4,"memory_gb":16,"gpu_count":0,"local_ssd_gb":100},"min_nodes":1,"max_nodes":5,"autoscaling_enabled":true},
            {"id":"np_b","az":"region-home-b","cell_id":"cell-region-home-b-001","subnet_id":"oyatie:cloud:region-home:ten_alpha:subnet:prod-b","security_groups":["sg_np_b_app","sg_np_b_web"],"flavor":{"class":"general_purpose","vcpu":4,"memory_gb":16,"gpu_count":0,"local_ssd_gb":100},"min_nodes":1,"max_nodes":5,"autoscaling_enabled":true},
            {"id":"np_c","az":"region-home-c","cell_id":"cell-region-home-c-001","subnet_id":"oyatie:cloud:region-home:ten_alpha:subnet:prod-c","security_groups":["sg_np_c_app","sg_np_c_web"],"flavor":{"class":"general_purpose","vcpu":4,"memory_gb":16,"gpu_count":0,"local_ssd_gb":100},"min_nodes":1,"max_nodes":5,"autoscaling_enabled":true}
        ],
        "residency": "strict_home_region",
        "data_class": "PUBLIC"
    }]);
    assert_eq!(encoded, expected);
}

#[test]
fn pending_intent_serialization_excludes_ephemeral_admission_claims() {
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
}

#[test]
fn invalid_singleton_data_class_is_rejected_separately_from_identity() {
    let mut invalid_class = pending_intent();
    invalid_class.data_class = "INTERNAL_ONLY".into();
    assert_eq!(
        validate_cloud_compute_k8s_create_intent(&invalid_class),
        Err(CloudComputeK8sApiError::Compute(
            CloudComputeError::InvalidDataClass
        )),
    );
}

#[test]
fn unknown_acceptance_contract_does_not_decode() {
    assert!(
        serde_json::from_str::<CloudComputeK8sAcceptanceContract>("\"unknown_contract\"").is_err()
    );
}
