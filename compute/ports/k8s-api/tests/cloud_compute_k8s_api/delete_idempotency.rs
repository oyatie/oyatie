#[test]
fn k8s_delete_api_rejects_reused_key_for_different_cluster() {
    let (catalog, _) = catalog_with_cluster();
    let mut delete_ledger = CloudComputeK8sDeleteIdempotencyLedger::default();

    delete_cloud_compute_k8s_cluster_from_api(
        &catalog,
        &mut delete_ledger,
        delete_request("req-del-reuse-1", "idem-del-reuse"),
    )
    .expect("initial delete succeeds");

    let mut drifted = delete_request("req-del-reuse-2", "idem-del-reuse");
    drifted.path_cluster_id = "oyatie:cloud:region-home:ten_alpha:k8s:other".to_string();

    let error = delete_cloud_compute_k8s_cluster_from_api(&catalog, &mut delete_ledger, drifted)
        .expect_err("same key different cluster_id is rejected");

    assert_eq!(
        error,
        CloudComputeK8sApiError::IdempotencyKeyReused {
            idempotency_key: "idem-del-reuse".to_string(),
        }
    );
    assert_eq!(error.cluster_delete_status_code(), 422);
    assert_eq!(delete_ledger.len(), 1);
}

#[test]
fn k8s_delete_error_response_request_id_roundtrips_and_matches_create_shape() {
    let catalog = CloudComputeCatalog::default(); // empty — triggers ClusterNotFound
    let mut delete_ledger = CloudComputeK8sDeleteIdempotencyLedger::default();

    let error = delete_cloud_compute_k8s_cluster_from_api(
        &catalog,
        &mut delete_ledger,
        delete_request("req-del-shape-check", "idem-del-shape-check"),
    )
    .expect_err("missing cluster for shape test");

    let response = error.error_response("req-del-shape-check");

    // request_id echoed in error body — same field as create surface uses
    assert_eq!(response.error.request_id, "req-del-shape-check");
    // error body shape: code, message, details present
    assert!(!response.error.code.is_empty());
    assert!(!response.error.message.is_empty());
    assert!(!response.error.details.is_empty());
    assert_eq!(response.error.message_localized, None);
}

#[test]
fn k8s_create_idempotency_ledger_enforces_bounded_retention() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::with_max_entries(1);

    create_cloud_compute_k8s_cluster_from_api(
        &mut catalog,
        &mut ledger,
        request("req-k8s-bound-1", "idem-k8s-bound-1"),
    )
    .expect("first create succeeds");
    let mut second = request("req-k8s-bound-2", "idem-k8s-bound-2");
    second.path_cluster_id = "oyatie:cloud:region-home:ten_alpha:k8s:prod-bound-2".to_string();
    second.body.resource_id = second.path_cluster_id.clone();
    create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, second)
        .expect("second create succeeds");

    assert_eq!(ledger.len(), 1);

    let replay_error = create_cloud_compute_k8s_cluster_from_api(
        &mut catalog,
        &mut ledger,
        request("req-k8s-bound-replay", "idem-k8s-bound-1"),
    )
    .expect_err(
        "evicted idempotency key is no longer replayable and reaches duplicate resource guard",
    );

    assert_eq!(
        replay_error,
        CloudComputeK8sApiError::Compute(CloudComputeError::DuplicateKubernetesCluster)
    );
}

#[test]
fn k8s_delete_idempotency_ledger_enforces_bounded_retention() {
    let (catalog, _) = catalog_with_cluster();
    let mut ledger = CloudComputeK8sDeleteIdempotencyLedger::with_max_entries(1);

    delete_cloud_compute_k8s_cluster_from_api(
        &catalog,
        &mut ledger,
        delete_request("req-del-bound-1", "idem-del-bound-1"),
    )
    .expect("first delete succeeds");
    delete_cloud_compute_k8s_cluster_from_api(
        &catalog,
        &mut ledger,
        delete_request("req-del-bound-2", "idem-del-bound-2"),
    )
    .expect("second delete succeeds");

    assert_eq!(ledger.len(), 1);
}
