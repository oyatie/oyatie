#[test]
fn k8s_create_api_rejects_foreign_security_group_proof_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let mut request = request("req-compute-k8s-sg-proof", "idem-compute-k8s-sg-proof");
    request.body.node_pools[0].security_groups[0].tenant_id = "ten_other".to_string();

    let error = create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request)
        .expect_err("security group proof must match tenant boundary");

    assert!(matches!(
        error,
        CloudComputeK8sApiError::NodePoolSecurityGroupBindingMismatch { .. }
    ));
    assert_eq!(error.cluster_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.kubernetes_clusters().count(), 0);
}

#[test]
fn k8s_create_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let request = request("req-compute-k8s-idem", "idem-compute-k8s-idem");
    create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request.clone())
        .expect("initial create succeeds");

    let mut drifted = request;
    drifted.body.control_plane_version = "v1.31.0-oyatie.1".to_string();
    assert_eq!(
        create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, drifted),
        Err(CloudComputeK8sApiError::IdempotencyKeyReused {
            idempotency_key: "idem-compute-k8s-idem".to_string(),
        })
    );
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.kubernetes_clusters().count(), 1);
}

#[test]
fn k8s_create_api_maps_duplicate_cluster_to_conflict() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    create_cloud_compute_k8s_cluster_from_api(
        &mut catalog,
        &mut ledger,
        request(
            "req-compute-k8s-duplicate-a",
            "idem-compute-k8s-duplicate-a",
        ),
    )
    .expect("first cluster create succeeds");

    let error = create_cloud_compute_k8s_cluster_from_api(
        &mut catalog,
        &mut ledger,
        request(
            "req-compute-k8s-duplicate-b",
            "idem-compute-k8s-duplicate-b",
        ),
    )
    .expect_err("second cluster with same id conflicts");

    assert_eq!(
        error,
        CloudComputeK8sApiError::Compute(CloudComputeError::DuplicateKubernetesCluster)
    );
    assert_eq!(error.cluster_create_status_code(), 409);
    assert_eq!(ledger.len(), 2);
    assert_eq!(catalog.kubernetes_clusters().count(), 1);
}

#[test]
fn k8s_create_api_maps_invalid_cluster_shape_to_bad_request() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let mut request = request("req-compute-k8s-shape", "idem-compute-k8s-shape");
    request.body.node_pools.truncate(1);

    let error = create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request)
        .expect_err("HA cluster requires three AZs");

    assert_eq!(
        error,
        CloudComputeK8sApiError::Compute(CloudComputeError::KubernetesHaRequiresThreeAzs)
    );
    assert_eq!(error.cluster_create_status_code(), 400);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.kubernetes_clusters().count(), 0);
}

// ── Delete surface tests ──────────────────────────────────────────────────────
