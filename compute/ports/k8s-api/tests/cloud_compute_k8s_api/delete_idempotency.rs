#[tokio::test]
async fn k8s_delete_api_rejects_reused_key_for_different_cluster() {
    let repository = delete_repository_with_cluster().await;
    let second_api_replica = repository.clone();

    delete_cloud_compute_k8s_cluster_from_api(
        &repository,
        delete_request("req-del-reuse-1", "idem-del-reuse"),
    )
    .await
    .expect("initial delete succeeds");

    let mut drifted = delete_request("req-del-reuse-2", "idem-del-reuse");
    drifted.path_cluster_id = "oyatie:cloud:region-home:ten_alpha:k8s:other".to_string();

    let error = delete_cloud_compute_k8s_cluster_from_api(&second_api_replica, drifted)
        .await
        .expect_err("same key different cluster_id is rejected");

    assert_eq!(
        error,
        CloudComputeK8sApiError::IdempotencyKeyReused {
            idempotency_key: "idem-del-reuse".to_string(),
        }
    );
    assert_eq!(error.cluster_delete_status_code(), 422);
    assert_eq!(repository.delete_operation_count(), 1);
}

#[tokio::test]
async fn k8s_delete_error_response_request_id_roundtrips_and_matches_create_shape() {
    let repository = LifecycleTestRepository::default();

    let error = delete_cloud_compute_k8s_cluster_from_api(
        &repository,
        delete_request("req-del-shape-check", "idem-del-shape-check"),
    )
    .await
    .expect_err("missing cluster for shape test");

    let response = error.error_response("req-del-shape-check");

    assert_eq!(response.error.request_id, "req-del-shape-check");
    assert!(!response.error.code.is_empty());
    assert!(!response.error.message.is_empty());
    assert!(!response.error.details.is_empty());
    assert_eq!(response.error.message_localized, None);
}

#[tokio::test]
async fn k8s_create_repository_replays_old_receipt_after_unrelated_operations_and_restore() {
    let repository = LifecycleTestRepository::default();

    let first = create_cloud_compute_k8s_cluster_from_api(
        &repository,
        request("req-k8s-durable-1", "idem-k8s-durable-1"),
    )
    .await
    .expect("first create succeeds");
    let mut second = request("req-k8s-durable-2", "idem-k8s-durable-2");
    second.path_cluster_id =
        "oyatie:cloud:region-home:ten_alpha:k8s:prod-durable-2".to_string();
    second.body.resource_id = second.path_cluster_id.clone();
    create_cloud_compute_k8s_cluster_from_api(&repository, second)
        .await
        .expect("second create succeeds");
    let restored = LifecycleTestRepository::from_restored_snapshot(repository.snapshot());

    let replay = create_cloud_compute_k8s_cluster_from_api(
        &restored,
        request("req-k8s-durable-replay", "idem-k8s-durable-1"),
    )
    .await
    .expect("old idempotency receipt remains replayable after restore");

    assert_eq!(replay, first);
    assert_eq!(restored.create_operation_count(), 2);
    assert_eq!(restored.cluster_count(), 2);
}

#[tokio::test]
async fn k8s_delete_repository_port_replays_from_restored_snapshot() {
    let repository = delete_repository_with_cluster().await;

    let first = delete_cloud_compute_k8s_cluster_from_api(
        &repository,
        delete_request("req-del-restart-1", "idem-del-restart"),
    )
    .await
    .expect("first delete succeeds");
    let restored = LifecycleTestRepository::from_restored_snapshot(repository.snapshot());

    let second = delete_cloud_compute_k8s_cluster_from_api(
        &restored,
        delete_request("req-del-restart-2", "idem-del-restart"),
    )
    .await
    .expect("repository port receipt replays from restored state");

    assert_eq!(second, first);
    assert_eq!(restored.delete_operation_count(), 1);
    assert_eq!(
        stored_cluster_lifecycle(&restored),
        ("creating".to_string(), "deleted".to_string())
    );
}

#[tokio::test]
async fn k8s_delete_api_rejects_unbound_repository_receipts_without_disclosure() {
    for case in ["resource", "tenant", "desired_state"] {
        let repository = delete_repository_with_cluster().await;
        let before = repository.snapshot();
        let mut cluster = repository
            .cluster_record(CLUSTER_ID)
            .expect("setup cluster exists");
        match case {
            "resource" => {
                cluster.resource_id =
                    "oyatie:cloud:region-home:ten_beta:k8s:foreign".to_string();
            }
            "tenant" => cluster.tenant_id = "ten_beta".to_string(),
            "desired_state" => cluster.desired_state = "present".to_string(),
            _ => unreachable!("test case is exhaustive"),
        }
        repository.return_next_delete_receipt(CloudComputeK8sDeleteReceipt {
            cluster,
            request_id: format!("repo-receipt-{case}"),
        });

        let error = delete_cloud_compute_k8s_cluster_from_api(
            &repository,
            delete_request(
                &format!("req-del-receipt-{case}"),
                &format!("idem-del-receipt-{case}"),
            ),
        )
        .await
        .expect_err("unbound repository receipt fails closed");

        assert_eq!(
            error,
            CloudComputeK8sApiError::LifecycleRepositoryInvariantViolation
        );
        assert_eq!(error.cluster_delete_status_code(), 503);
        let response = error.error_response(format!("req-del-receipt-{case}"));
        assert_eq!(
            response.error.code,
            "CLOUD_COMPUTE_K8S_LIFECYCLE_REPOSITORY_UNAVAILABLE"
        );
        let rendered = format!("{response:?}");
        assert!(!rendered.contains("ten_beta"));
        assert!(!rendered.contains("foreign"));
        assert_eq!(repository.snapshot(), before);
    }
}

#[tokio::test]
async fn k8s_delete_repository_failure_before_commit_is_atomic_and_retryable() {
    let repository = delete_repository_with_cluster().await;
    let before = repository.snapshot();
    repository.fail_next_commit();

    let error = delete_cloud_compute_k8s_cluster_from_api(
        &repository,
        delete_request("req-del-failed-commit", "idem-del-failed-commit"),
    )
    .await
    .expect_err("failure between staging and commit fails closed");

    assert_eq!(error, CloudComputeK8sApiError::LifecycleRepositoryUnavailable);
    assert_eq!(error.cluster_delete_status_code(), 503);
    assert_eq!(
        error
            .error_response("req-del-failed-commit")
            .error
            .retry_after_seconds,
        Some(1)
    );
    assert_eq!(repository.snapshot(), before);

    let response = delete_cloud_compute_k8s_cluster_from_api(
        &repository,
        delete_request("req-del-retry-commit", "idem-del-failed-commit"),
    )
    .await
    .expect("retry can commit the intact operation");
    assert_eq!(response.data.state, "creating");
    assert_eq!(response.data.desired_state, "deleted");
    assert_eq!(repository.delete_operation_count(), 1);
}
