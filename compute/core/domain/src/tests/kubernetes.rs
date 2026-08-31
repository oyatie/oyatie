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
    assert_eq!(
        cluster.desired_state.value,
        KubernetesClusterDesiredState::Present
    );
    assert_eq!(
        cluster.schema_version.value,
        KUBERNETES_CLUSTER_SCHEMA_VERSION
    );
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

#[test]
fn kubernetes_reconciliation_is_deterministic_across_lifecycle_states() {
    for observed_state in [
        KubernetesClusterState::Creating,
        KubernetesClusterState::Reconciling,
    ] {
        assert_eq!(
            reconcile_kubernetes_cluster(KubernetesClusterReconcileInput {
                desired_state: KubernetesClusterDesiredState::Present,
                observation: KubernetesClusterObservation::Known(observed_state),
            }),
            Ok(KubernetesClusterReconcileAction::AwaitObservation)
        );
    }
    assert_eq!(
        reconcile_kubernetes_cluster(KubernetesClusterReconcileInput {
            desired_state: KubernetesClusterDesiredState::Present,
            observation: KubernetesClusterObservation::Known(KubernetesClusterState::Ready),
        }),
        Ok(KubernetesClusterReconcileAction::Noop)
    );

    for observed_state in [
        KubernetesClusterState::Creating,
        KubernetesClusterState::Ready,
        KubernetesClusterState::Reconciling,
    ] {
        assert_eq!(
            reconcile_kubernetes_cluster(KubernetesClusterReconcileInput {
                desired_state: KubernetesClusterDesiredState::Deleted,
                observation: KubernetesClusterObservation::Known(observed_state),
            }),
            Ok(KubernetesClusterReconcileAction::BeginDraining)
        );
    }

    assert_eq!(
        reconcile_kubernetes_cluster(KubernetesClusterReconcileInput {
            desired_state: KubernetesClusterDesiredState::Deleted,
            observation: KubernetesClusterObservation::Known(KubernetesClusterState::Draining),
        }),
        Ok(KubernetesClusterReconcileAction::ActuateDeletion)
    );
    assert_eq!(
        reconcile_kubernetes_cluster(KubernetesClusterReconcileInput {
            desired_state: KubernetesClusterDesiredState::Deleted,
            observation: KubernetesClusterObservation::Known(KubernetesClusterState::Deleted),
        }),
        Ok(KubernetesClusterReconcileAction::Noop)
    );
}

#[test]
fn kubernetes_reconciliation_fails_closed_for_unknown_or_inconsistent_observation() {
    assert_eq!(
        reconcile_kubernetes_cluster(KubernetesClusterReconcileInput {
            desired_state: KubernetesClusterDesiredState::Deleted,
            observation: KubernetesClusterObservation::Unknown,
        }),
        Err(KubernetesClusterReconcileError::UnknownObservation)
    );
    assert_eq!(
        reconcile_kubernetes_cluster(KubernetesClusterReconcileInput {
            desired_state: KubernetesClusterDesiredState::Present,
            observation: KubernetesClusterObservation::Known(KubernetesClusterState::Draining),
        }),
        Err(KubernetesClusterReconcileError::InconsistentLifecycle {
            desired_state: KubernetesClusterDesiredState::Present,
            observed_state: KubernetesClusterState::Draining,
        })
    );
}

#[test]
fn catalog_persists_idempotent_kubernetes_deletion_intent() {
    let mut catalog = CloudComputeCatalog::default();
    let created = catalog
        .create_kubernetes_cluster(k8s_create())
        .expect("cluster contract is valid");
    let cluster_id = created.resource_id.value.clone();

    let first = catalog
        .request_kubernetes_cluster_deletion(&cluster_id)
        .expect("known cluster deletion intent is accepted");
    let replay = catalog
        .request_kubernetes_cluster_deletion(&cluster_id)
        .expect("repeated deletion intent is idempotent");

    assert_eq!(first, replay);
    assert_eq!(first.state.value, KubernetesClusterState::Creating);
    assert_eq!(
        first.desired_state.value,
        KubernetesClusterDesiredState::Deleted
    );
    assert_eq!(
        reconcile_kubernetes_cluster(KubernetesClusterReconcileInput {
            desired_state: first.desired_state.value,
            observation: KubernetesClusterObservation::Known(first.state.value),
        }),
        Ok(KubernetesClusterReconcileAction::BeginDraining)
    );
    assert_eq!(
        catalog
            .kubernetes_clusters()
            .find(|cluster| cluster.resource_id.value == cluster_id)
            .expect("mutated cluster remains addressable"),
        &first
    );
}

#[test]
fn missing_cluster_deletion_intent_leaves_catalog_unchanged() {
    let mut catalog = CloudComputeCatalog::default();
    let before = catalog.clone();
    let missing = compute_resource::ResourceId::new(
        "oyatie:cloud:region-alpha:ten_alpha:k8s:missing".to_string(),
    )
    .expect("missing cluster id is structurally valid");

    assert_eq!(
        catalog.request_kubernetes_cluster_deletion(&missing),
        Err(KubernetesClusterMutationError::UnknownCluster)
    );
    assert_eq!(catalog, before);
}
