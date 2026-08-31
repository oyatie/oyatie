#[test]
fn k8s_delete_api_rejects_reused_key_for_different_cluster() {
    let mut repository = delete_repository_with_cluster();
    let mut second_api_replica = repository.clone();

    delete_cloud_compute_k8s_cluster_from_api(
        &mut repository,
        delete_request("req-del-reuse-1", "idem-del-reuse"),
    )
    .expect("initial delete succeeds");

    let mut drifted = delete_request("req-del-reuse-2", "idem-del-reuse");
    drifted.path_cluster_id = "oyatie:cloud:region-home:ten_alpha:k8s:other".to_string();

    let error = delete_cloud_compute_k8s_cluster_from_api(&mut second_api_replica, drifted)
        .expect_err("same key different cluster_id is rejected");

    assert_eq!(
        error,
        CloudComputeK8sApiError::IdempotencyKeyReused {
            idempotency_key: "idem-del-reuse".to_string(),
        }
    );
    assert_eq!(error.cluster_delete_status_code(), 422);
    assert_eq!(repository.entry_count(), 1);
}

#[test]
fn k8s_delete_error_response_request_id_roundtrips_and_matches_create_shape() {
    let mut repository = DeleteTestRepository::new(CloudComputeCatalog::default());

    let error = delete_cloud_compute_k8s_cluster_from_api(
        &mut repository,
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
fn k8s_delete_repository_port_replays_from_restored_snapshot() {
    let mut repository = delete_repository_with_cluster();

    let first = delete_cloud_compute_k8s_cluster_from_api(
        &mut repository,
        delete_request("req-del-restart-1", "idem-del-restart"),
    )
    .expect("first delete succeeds");
    let mut restored = DeleteTestRepository::from_restored_snapshot(repository.snapshot());

    let second = delete_cloud_compute_k8s_cluster_from_api(
        &mut restored,
        delete_request("req-del-restart-2", "idem-del-restart"),
    )
    .expect("repository port receipt replays from restored state");

    assert_eq!(second, first);
    assert_eq!(restored.entry_count(), 1);
    assert_eq!(
        stored_cluster_lifecycle(&restored),
        (
            KubernetesClusterState::Creating,
            KubernetesClusterDesiredState::Deleted,
        )
    );
}

#[test]
fn k8s_delete_api_rejects_unbound_repository_receipts_without_disclosure() {
    for case in ["resource", "tenant", "desired_state"] {
        let mut repository = delete_repository_with_cluster();
        let before = repository.snapshot();
        let mut cluster = repository
            .catalog_snapshot()
            .kubernetes_clusters()
            .next()
            .expect("setup cluster exists")
            .clone();
        match case {
            "resource" => {
                cluster.resource_id.value = compute_resource::ResourceId::new(
                    "oyatie:cloud:region-home:ten_beta:k8s:foreign".to_string(),
                )
                .expect("foreign cluster id is structurally valid");
            }
            "tenant" => cluster.tenant_id.value = "ten_beta".to_string(),
            "desired_state" => {
                cluster.desired_state.value = KubernetesClusterDesiredState::Present;
            }
            _ => unreachable!("test case is exhaustive"),
        }
        repository.return_next_receipt(CloudComputeK8sDeleteReceipt {
            cluster,
            request_id: format!("repo-receipt-{case}"),
        });

        let error = delete_cloud_compute_k8s_cluster_from_api(
            &mut repository,
            delete_request(
                &format!("req-del-receipt-{case}"),
                &format!("idem-del-receipt-{case}"),
            ),
        )
        .expect_err("unbound repository receipt fails closed");

        assert_eq!(
            error,
            CloudComputeK8sApiError::DeletionRepositoryInvariantViolation
        );
        assert_eq!(error.cluster_delete_status_code(), 503);
        let response = error.error_response(format!("req-del-receipt-{case}"));
        assert_eq!(
            response.error.code,
            "CLOUD_COMPUTE_K8S_DELETION_REPOSITORY_UNAVAILABLE"
        );
        let rendered = format!("{response:?}");
        assert!(!rendered.contains("ten_beta"));
        assert!(!rendered.contains("foreign"));
        assert_eq!(repository.snapshot(), before);
    }
}

#[test]
fn k8s_delete_repository_failure_before_commit_is_atomic_and_retryable() {
    let mut repository = delete_repository_with_cluster();
    let before = repository.snapshot();
    repository.fail_next_commit();

    let error = delete_cloud_compute_k8s_cluster_from_api(
        &mut repository,
        delete_request("req-del-failed-commit", "idem-del-failed-commit"),
    )
    .expect_err("failure between staging and commit fails closed");

    assert_eq!(error, CloudComputeK8sApiError::DeletionRepositoryUnavailable);
    assert_eq!(error.cluster_delete_status_code(), 503);
    assert_eq!(
        error.error_response("req-del-failed-commit")
            .error
            .retry_after_seconds,
        Some(1)
    );
    assert_eq!(repository.snapshot(), before);

    let response = delete_cloud_compute_k8s_cluster_from_api(
        &mut repository,
        delete_request("req-del-retry-commit", "idem-del-failed-commit"),
    )
    .expect("retry can commit the intact operation");
    assert_eq!(response.data.state, "creating");
    assert_eq!(response.data.desired_state, "deleted");
    assert_eq!(repository.entry_count(), 1);
}
