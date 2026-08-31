#[test]
fn api_surface_status_contracts_are_covered() {
    assert_eq!(
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
        "cloud.compute.k8s.cluster.create"
    );
    assert_eq!(CloudComputeK8sClusterCreateApiStatus::Created.code(), 201);
    assert_eq!(
        CloudComputeK8sClusterCreateApiStatus::BadRequest.code(),
        400
    );
    assert_eq!(
        CloudComputeK8sClusterCreateApiStatus::Unauthorized.code(),
        401
    );
    assert_eq!(CloudComputeK8sClusterCreateApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudComputeK8sClusterCreateApiStatus::NotFound.code(), 404);
    assert_eq!(CloudComputeK8sClusterCreateApiStatus::Conflict.code(), 409);
    assert_eq!(
        CloudComputeK8sClusterCreateApiStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn k8s_create_api_creates_cluster_once_and_replays_same_idempotent_result() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let request = request("req-compute-k8s-create", "idem-compute-k8s-create");

    let first =
        create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request.clone())
            .expect("authorized cluster create succeeds");
    let second = create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.kubernetes_clusters().count(), 1);
    assert_eq!(first.metadata.request_id, "req-compute-k8s-create");
    assert_eq!(first.data.resource_id, CLUSTER_ID);
    assert_eq!(first.data.tenant_id, "ten_alpha");
    assert_eq!(first.data.region, "region-home");
    assert_eq!(first.data.flavor, "high_availability");
    assert_eq!(first.data.control_plane_version, "v1.30.2-oyatie.1");
    assert!(first.data.control_plane_private);
    assert_eq!(first.data.node_pool_count, 3);
    assert_eq!(first.data.residency, "strict_home_region");
    assert_eq!(first.data.state, "creating");
    assert_eq!(first.data.data_class, "PUBLIC");
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn planned_create_cluster_entrypoint_delegates_to_api_create() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let request = request(
        "req-compute-k8s-create-alias",
        "idem-compute-k8s-create-alias",
    );

    let response = create_cluster(&mut catalog, &mut ledger, request)
        .expect("stable planned create_cluster entrypoint succeeds");

    assert_eq!(response.metadata.request_id, "req-compute-k8s-create-alias");
    assert_eq!(response.data.resource_id, CLUSTER_ID);
    assert_eq!(response.data.state, "creating");
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.kubernetes_clusters().count(), 1);
}

#[test]
fn k8s_create_api_rejects_path_body_drift_before_catalog_mutation() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let mut request = request("req-compute-k8s-drift", "idem-compute-k8s-drift");
    request.body.resource_id = "oyatie:cloud:region-home:ten_alpha:k8s:other".to_string();

    let error = create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request)
        .expect_err("path/body cluster drift is rejected");

    assert_eq!(
        error,
        CloudComputeK8sApiError::ClusterIdMismatch {
            path_cluster_id: CLUSTER_ID.to_string(),
            body_resource_id: "oyatie:cloud:region-home:ten_alpha:k8s:other".to_string(),
        }
    );
    assert_eq!(error.cluster_create_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(catalog.kubernetes_clusters().count(), 0);
}

#[test]
fn k8s_create_api_legacy_entrypoint_fails_closed_without_authorization_verifier() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let missing_verifier_request = request(
        "req-compute-k8s-missing-verifier",
        "idem-compute-k8s-missing-verifier",
    );

    let error = create_cloud_compute_k8s_cluster_from_api_without_authorization_verifier(
        &mut catalog,
        &mut ledger,
        missing_verifier_request,
    )
    .expect_err("legacy create entrypoint has no trusted authorization verifier");

    assert_eq!(
        error,
        CloudComputeK8sApiError::AuthorizationDenied {
            surface: CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE.to_string(),
        }
    );
    assert_eq!(error.cluster_create_status_code(), 403);
    let planned_error = create_cluster_without_authorization_verifier(
        &mut catalog,
        &mut ledger,
        request(
            "req-compute-k8s-planned-missing-verifier",
            "idem-compute-k8s-planned-missing-verifier",
        ),
    )
    .expect_err("legacy planned create entrypoint has no trusted authorization verifier");
    assert_eq!(planned_error, error);
    assert!(ledger.is_empty());
    assert_eq!(catalog.kubernetes_clusters().count(), 0);
}
