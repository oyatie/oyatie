// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use compute_domain::{CloudComputeCatalog, CloudComputeError};
use compute_k8s_api::{
    CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE, CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
    CloudComputeK8sApiAuthorization, CloudComputeK8sApiAuthorizationProof,
    CloudComputeK8sApiBoundaryContext, CloudComputeK8sApiError, CloudComputeK8sApiPrincipal,
    CloudComputeK8sClusterCreateApiRequest, CloudComputeK8sClusterCreateApiStatus,
    CloudComputeK8sClusterCreateRequest, CloudComputeK8sClusterCreateSuccessResponse,
    CloudComputeK8sClusterDeleteApiRequest, CloudComputeK8sClusterDeleteApiStatus,
    CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sCreateIdempotencyLedger,
    CloudComputeK8sDeleteIdempotencyLedger, CloudComputeK8sNodePoolCreateRequest,
    CloudComputeK8sNodePoolFlavorSpec, CloudComputeK8sQuotaEnvelope,
    CloudComputeK8sSecurityGroupRef, CloudComputeK8sTrustedAuthorizationVerifier,
    create_cloud_compute_k8s_cluster_from_api as create_cloud_compute_k8s_cluster_from_api_without_authorization_verifier,
    create_cloud_compute_k8s_cluster_from_api_with_authorization_verifier,
    create_cluster as create_cluster_without_authorization_verifier,
    create_cluster_with_authorization_verifier,
    delete_cloud_compute_k8s_cluster_from_api as delete_cloud_compute_k8s_cluster_from_api_without_authorization_verifier,
    delete_cloud_compute_k8s_cluster_from_api_with_authorization_verifier,
    delete_cluster as delete_cluster_without_authorization_verifier,
    delete_cluster_with_authorization_verifier,
};

const CLUSTER_ID: &str = "oyatie:cloud:region-home:ten_alpha:k8s:prod";
const K8S_AUTHZ_EVALUATION_EPOCH_SECONDS: u64 = 1_700_099_500;

fn boundary_for(request_id: &str, idempotency_key: &str) -> CloudComputeK8sApiBoundaryContext {
    CloudComputeK8sApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudComputeK8sApiPrincipal {
    CloudComputeK8sApiPrincipal {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization_for(principal_id: &str, surfaces: &[&str]) -> CloudComputeK8sApiAuthorization {
    let decision_id = format!("authz_decision_{principal_id}");
    CloudComputeK8sApiAuthorization {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: decision_id.clone(),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
        proof: Some(authorization_proof_for(
            principal_id,
            CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
            &decision_id,
        )),
    }
}
fn authorization_proof_for(
    principal_id: &str,
    surface: &str,
    decision_id: &str,
) -> CloudComputeK8sApiAuthorizationProof {
    CloudComputeK8sApiAuthorizationProof {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
        surface: surface.to_string(),
        decision_id: decision_id.to_string(),
        verified: true,
        issued_at_epoch_seconds: 1_700_099_000,
        expires_at_epoch_seconds: 1_700_100_000,
    }
}

fn quota() -> CloudComputeK8sQuotaEnvelope {
    CloudComputeK8sQuotaEnvelope {
        vcpu_limit: 128,
        memory_gb_limit: 512,
        gpu_limit: 8,
        local_ssd_gb_limit: 4_096,
        current_vcpu: 4,
        current_memory_gb: 16,
        current_gpu: 0,
        current_local_ssd_gb: 100,
    }
}

fn node_flavor() -> CloudComputeK8sNodePoolFlavorSpec {
    CloudComputeK8sNodePoolFlavorSpec {
        class: "general_purpose".to_string(),
        vcpu: 4,
        memory_gb: 16,
        gpu_count: 0,
        local_ssd_gb: 100,
    }
}

fn node_pool(id: &str, az: &str, subnet: &str) -> CloudComputeK8sNodePoolCreateRequest {
    CloudComputeK8sNodePoolCreateRequest {
        id: id.to_string(),
        az: az.to_string(),
        cell_id: format!("cell-{az}-001"),
        subnet_id: subnet.to_string(),
        security_groups: vec![
            CloudComputeK8sSecurityGroupRef {
                value: format!("sg_{id}_web"),
                tenant_id: "ten_alpha".to_string(),
                region: "region-home".to_string(),
                subnet_id: subnet.to_string(),
            },
            CloudComputeK8sSecurityGroupRef {
                value: format!("sg_{id}_app"),
                tenant_id: "ten_alpha".to_string(),
                region: "region-home".to_string(),
                subnet_id: subnet.to_string(),
            },
        ],
        flavor: node_flavor(),
        min_nodes: 1,
        max_nodes: 5,
        autoscaling_enabled: true,
    }
}

fn body(resource_id: &str) -> CloudComputeK8sClusterCreateRequest {
    CloudComputeK8sClusterCreateRequest {
        resource_id: resource_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        region: "region-home".to_string(),
        flavor: "high_availability".to_string(),
        control_plane_version: "v1.30.2-oyatie.1".to_string(),
        control_plane_private: true,
        node_pools: vec![
            node_pool(
                "np_a",
                "region-home-a",
                "oyatie:cloud:region-home:ten_alpha:subnet:prod-a",
            ),
            node_pool(
                "np_b",
                "region-home-b",
                "oyatie:cloud:region-home:ten_alpha:subnet:prod-b",
            ),
            node_pool(
                "np_c",
                "region-home-c",
                "oyatie:cloud:region-home:ten_alpha:subnet:prod-c",
            ),
        ],
        quota: quota(),
        residency: "strict_home_region".to_string(),
        data_class: "PUBLIC".to_string(),
        created_at_epoch_seconds: 1_700_100_010,
    }
}

fn request(request_id: &str, idempotency_key: &str) -> CloudComputeK8sClusterCreateApiRequest {
    CloudComputeK8sClusterCreateApiRequest {
        path_cluster_id: CLUSTER_ID.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_compute"),
        authorization: authorization_for("sp_compute", &[CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE]),
        body: body(CLUSTER_ID),
    }
}

fn trusted_create_verifier_for(
    request: &CloudComputeK8sClusterCreateApiRequest,
) -> CloudComputeK8sTrustedAuthorizationVerifier {
    CloudComputeK8sTrustedAuthorizationVerifier::new(K8S_AUTHZ_EVALUATION_EPOCH_SECONDS)
        .with_authorization_proof(authorization_proof_for(
            &request.principal.principal_id,
            CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
            &request.authorization.decision_id,
        ))
}

fn create_cloud_compute_k8s_cluster_from_api(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sCreateIdempotencyLedger,
    request: CloudComputeK8sClusterCreateApiRequest,
) -> Result<CloudComputeK8sClusterCreateSuccessResponse, CloudComputeK8sApiError> {
    let verifier = trusted_create_verifier_for(&request);
    create_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        catalog,
        idempotency_ledger,
        request,
        &verifier,
    )
}

fn create_cluster(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sCreateIdempotencyLedger,
    request: CloudComputeK8sClusterCreateApiRequest,
) -> Result<CloudComputeK8sClusterCreateSuccessResponse, CloudComputeK8sApiError> {
    let verifier = trusted_create_verifier_for(&request);
    create_cluster_with_authorization_verifier(catalog, idempotency_ledger, request, &verifier)
}

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

#[test]
fn k8s_create_api_rejects_trusted_verifier_mismatches_before_ledger() {
    for case in [
        "forged",
        "tenant",
        "principal",
        "surface",
        "decision",
        "expired",
        "stale",
    ] {
        let mut catalog = CloudComputeCatalog::default();
        let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
        let request = request(
            &format!("req-compute-k8s-authz-{case}"),
            &format!("idem-compute-k8s-authz-{case}"),
        );
        let mut proof = authorization_proof_for(
            "sp_compute",
            CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
            &request.authorization.decision_id,
        );
        let verifier_key = match case {
            "forged" => {
                proof.verified = false;
                None
            }
            "tenant" => {
                proof.tenant_id = "ten_other".to_string();
                None
            }
            "principal" => {
                proof.principal_id = "sp_other".to_string();
                None
            }
            "surface" => {
                proof.surface = "cloud.compute.vm.create".to_string();
                None
            }
            "decision" => {
                proof.decision_id = "authz_decision_other".to_string();
                Some(request.authorization.decision_id.clone())
            }
            "expired" => {
                proof.expires_at_epoch_seconds = proof.issued_at_epoch_seconds;
                None
            }
            "stale" => {
                proof.expires_at_epoch_seconds = K8S_AUTHZ_EVALUATION_EPOCH_SECONDS;
                None
            }
            _ => unreachable!("test case is exhaustive"),
        };
        let mut verifier =
            CloudComputeK8sTrustedAuthorizationVerifier::new(K8S_AUTHZ_EVALUATION_EPOCH_SECONDS);
        if let Some(decision_id) = verifier_key {
            verifier.trust_authorization_proof_for_decision(decision_id, proof);
        } else {
            verifier.trust_authorization_proof(proof);
        }

        let error = create_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
            &mut catalog,
            &mut ledger,
            request,
            &verifier,
        )
        .expect_err("trusted verifier mismatch is rejected before mutation");

        assert_eq!(
            error,
            CloudComputeK8sApiError::AuthorizationDenied {
                surface: CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE.to_string(),
            }
        );
        assert_eq!(error.cluster_create_status_code(), 403);
        assert!(ledger.is_empty());
        assert_eq!(catalog.kubernetes_clusters().count(), 0);
    }
}

#[test]
fn k8s_create_api_ignores_caller_supplied_authorization_proof() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let mut request = request(
        "req-compute-k8s-ignore-proof",
        "idem-compute-k8s-ignore-proof",
    );
    request.authorization.tenant_id = "ten_forged".to_string();
    request.authorization.principal_id = "sp_forged_compute".to_string();
    request.authorization.allowed_surfaces.clear();
    request.authorization.proof = Some(CloudComputeK8sApiAuthorizationProof {
        tenant_id: "ten_other".to_string(),
        principal_id: "sp_other".to_string(),
        surface: "cloud.compute.vm.create".to_string(),
        decision_id: "authz_decision_other".to_string(),
        verified: false,
        issued_at_epoch_seconds: 1_700_099_000,
        expires_at_epoch_seconds: 1_700_099_000,
    });
    let verifier = trusted_create_verifier_for(&request);

    let response = create_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        &mut catalog,
        &mut ledger,
        request,
        &verifier,
    )
    .expect("trusted verifier state, not caller proof fields, authorizes create");

    assert_eq!(response.data.resource_id, CLUSTER_ID);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.kubernetes_clusters().count(), 1);
}

#[test]
fn k8s_create_api_separates_missing_authentication_from_denied_authorization() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let mut request = request("req-compute-k8s-authn", "idem-compute-k8s-authn");
    request.principal.principal_id.clear();

    let error = create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request)
        .expect_err("missing authenticated principal is an authentication failure");

    assert_eq!(error, CloudComputeK8sApiError::EmptyPrincipalId);
    assert_eq!(error.cluster_create_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.kubernetes_clusters().count(), 0);
}

#[test]
fn k8s_create_api_replays_with_refreshed_authz_and_reordered_pools() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let request = request(
        "req-compute-k8s-authz-refresh-1",
        "idem-compute-k8s-authz-refresh",
    );
    let first =
        create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request.clone())
            .expect("initial cluster create succeeds");

    let mut retry = request;
    retry.boundary.request_id = "req-compute-k8s-authz-refresh-2".to_string();
    retry.authorization.decision_id = "authz_decision_sp_compute_refreshed".to_string();
    retry.authorization.proof = Some(authorization_proof_for(
        "sp_compute",
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
        &retry.authorization.decision_id,
    ));
    retry.authorization.allowed_surfaces = vec![
        "cloud.compute.vm.create".to_string(),
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE.to_string(),
    ];
    retry.body.node_pools.reverse();
    for pool in &mut retry.body.node_pools {
        pool.security_groups.reverse();
    }
    let second = create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, retry)
        .expect("refreshed authorization evidence does not change operation fingerprint");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.kubernetes_clusters().count(), 1);
}

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

fn delete_boundary_for(
    request_id: &str,
    idempotency_key: &str,
) -> CloudComputeK8sApiBoundaryContext {
    CloudComputeK8sApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn delete_authorization_for(
    principal_id: &str,
    surfaces: &[&str],
) -> CloudComputeK8sApiAuthorization {
    let decision_id = format!("authz_del_{principal_id}");
    CloudComputeK8sApiAuthorization {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: decision_id.clone(),
        allowed_surfaces: surfaces.iter().map(|s| (*s).to_string()).collect(),
        proof: Some(authorization_proof_for(
            principal_id,
            CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
            &decision_id,
        )),
    }
}

fn delete_request(
    request_id: &str,
    idempotency_key: &str,
) -> CloudComputeK8sClusterDeleteApiRequest {
    CloudComputeK8sClusterDeleteApiRequest {
        path_cluster_id: CLUSTER_ID.to_string(),
        boundary: delete_boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_compute"),
        authorization: delete_authorization_for(
            "sp_compute",
            &[CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE],
        ),
    }
}

fn trusted_delete_verifier_for(
    request: &CloudComputeK8sClusterDeleteApiRequest,
) -> CloudComputeK8sTrustedAuthorizationVerifier {
    CloudComputeK8sTrustedAuthorizationVerifier::new(K8S_AUTHZ_EVALUATION_EPOCH_SECONDS)
        .with_authorization_proof(authorization_proof_for(
            &request.principal.principal_id,
            CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
            &request.authorization.decision_id,
        ))
}

fn delete_cloud_compute_k8s_cluster_from_api(
    catalog: &CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sDeleteIdempotencyLedger,
    request: CloudComputeK8sClusterDeleteApiRequest,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    let verifier = trusted_delete_verifier_for(&request);
    delete_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        catalog,
        idempotency_ledger,
        request,
        &verifier,
    )
}

fn delete_cluster(
    catalog: &CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeK8sDeleteIdempotencyLedger,
    request: CloudComputeK8sClusterDeleteApiRequest,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    let verifier = trusted_delete_verifier_for(&request);
    delete_cluster_with_authorization_verifier(catalog, idempotency_ledger, request, &verifier)
}

/// Populate the catalog with one cluster so delete tests have something to find.
fn catalog_with_cluster() -> (CloudComputeCatalog, CloudComputeK8sCreateIdempotencyLedger) {
    let mut catalog = CloudComputeCatalog::default();
    let mut create_ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    create_cloud_compute_k8s_cluster_from_api(
        &mut catalog,
        &mut create_ledger,
        request("req-setup-delete", "idem-setup-delete"),
    )
    .expect("setup cluster create succeeds");
    (catalog, create_ledger)
}

#[test]
fn k8s_delete_api_surface_constants_and_status_codes() {
    assert_eq!(
        CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
        "cloud.compute.k8s.cluster.delete"
    );
    assert_eq!(CloudComputeK8sClusterDeleteApiStatus::Accepted.code(), 202);
    assert_eq!(
        CloudComputeK8sClusterDeleteApiStatus::BadRequest.code(),
        400
    );
    assert_eq!(
        CloudComputeK8sClusterDeleteApiStatus::Unauthorized.code(),
        401
    );
    assert_eq!(CloudComputeK8sClusterDeleteApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudComputeK8sClusterDeleteApiStatus::NotFound.code(), 404);
    assert_eq!(
        CloudComputeK8sClusterDeleteApiStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn k8s_delete_api_accepts_valid_teardown_and_projects_draining_state() {
    let (catalog, _) = catalog_with_cluster();
    let mut delete_ledger = CloudComputeK8sDeleteIdempotencyLedger::default();

    let response = delete_cloud_compute_k8s_cluster_from_api(
        &catalog,
        &mut delete_ledger,
        delete_request("req-del-happy", "idem-del-happy"),
    )
    .expect("authorized cluster delete succeeds");

    assert_eq!(response.metadata.request_id, "req-del-happy");
    assert_eq!(response.data.resource_id, CLUSTER_ID);
    assert_eq!(response.data.tenant_id, "ten_alpha");
    assert_eq!(response.data.state, "draining");
    assert_eq!(delete_ledger.len(), 1);
}

#[test]
fn k8s_delete_api_replay_returns_same_response_without_double_teardown() {
    let (catalog, _) = catalog_with_cluster();
    let mut delete_ledger = CloudComputeK8sDeleteIdempotencyLedger::default();

    let first = delete_cloud_compute_k8s_cluster_from_api(
        &catalog,
        &mut delete_ledger,
        delete_request("req-del-idem-1", "idem-del-replay"),
    )
    .expect("first delete accepted");

    let mut retry = delete_request("req-del-idem-2", "idem-del-replay");
    retry.authorization.decision_id = "authz_del_sp_compute_refreshed".to_string();
    retry.authorization.proof = Some(authorization_proof_for(
        "sp_compute",
        CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
        &retry.authorization.decision_id,
    ));
    let second = delete_cloud_compute_k8s_cluster_from_api(&catalog, &mut delete_ledger, retry)
        .expect("same idempotency key replays");

    assert_eq!(first, second);
    assert_eq!(delete_ledger.len(), 1);
}

#[test]
fn k8s_delete_api_stable_entrypoint_delegates() {
    let (catalog, _) = catalog_with_cluster();
    let mut delete_ledger = CloudComputeK8sDeleteIdempotencyLedger::default();

    let response = delete_cluster(
        &catalog,
        &mut delete_ledger,
        delete_request("req-del-alias", "idem-del-alias"),
    )
    .expect("stable delete_cluster entrypoint succeeds");

    assert_eq!(response.metadata.request_id, "req-del-alias");
    assert_eq!(response.data.state, "draining");
    assert_eq!(delete_ledger.len(), 1);
}

#[test]
fn k8s_delete_api_rejects_empty_request_id() {
    let (catalog, _) = catalog_with_cluster();
    let mut delete_ledger = CloudComputeK8sDeleteIdempotencyLedger::default();
    let mut req = delete_request("req-del-empty-rid", "idem-del-empty-rid");
    req.boundary.request_id.clear();

    let error = delete_cloud_compute_k8s_cluster_from_api(&catalog, &mut delete_ledger, req)
        .expect_err("empty request_id rejected");

    assert_eq!(error, CloudComputeK8sApiError::EmptyRequestId);
    assert_eq!(error.cluster_delete_status_code(), 400);
    assert!(delete_ledger.is_empty());
}

#[test]
fn k8s_delete_api_rejects_empty_tenant() {
    let (catalog, _) = catalog_with_cluster();
    let mut delete_ledger = CloudComputeK8sDeleteIdempotencyLedger::default();
    let mut req = delete_request("req-del-empty-ten", "idem-del-empty-ten");
    req.boundary.tenant_id.clear();

    let error = delete_cloud_compute_k8s_cluster_from_api(&catalog, &mut delete_ledger, req)
        .expect_err("empty tenant rejected");

    assert_eq!(error, CloudComputeK8sApiError::EmptyTenantHeader);
    assert_eq!(error.cluster_delete_status_code(), 400);
    assert!(delete_ledger.is_empty());
}

#[test]
fn k8s_delete_api_rejects_empty_idempotency_key() {
    let (catalog, _) = catalog_with_cluster();
    let mut delete_ledger = CloudComputeK8sDeleteIdempotencyLedger::default();
    let mut req = delete_request("req-del-empty-idem", "idem-del-empty-idem");
    req.boundary.idempotency_key.clear();

    let error = delete_cloud_compute_k8s_cluster_from_api(&catalog, &mut delete_ledger, req)
        .expect_err("empty idempotency_key rejected");

    assert_eq!(error, CloudComputeK8sApiError::EmptyIdempotencyKey);
    assert_eq!(error.cluster_delete_status_code(), 400);
    assert!(delete_ledger.is_empty());
}

#[test]
fn k8s_delete_api_rejects_empty_principal_as_unauthorized() {
    let (catalog, _) = catalog_with_cluster();
    let mut delete_ledger = CloudComputeK8sDeleteIdempotencyLedger::default();
    let mut req = delete_request("req-del-empty-prin", "idem-del-empty-prin");
    req.principal.principal_id.clear();

    let error = delete_cloud_compute_k8s_cluster_from_api(&catalog, &mut delete_ledger, req)
        .expect_err("empty principal_id is 401");

    assert_eq!(error, CloudComputeK8sApiError::EmptyPrincipalId);
    assert_eq!(error.cluster_delete_status_code(), 401);
    assert!(delete_ledger.is_empty());
}

#[test]
fn k8s_delete_api_legacy_entrypoint_fails_closed_without_authorization_verifier() {
    let (catalog, _) = catalog_with_cluster();
    let mut delete_ledger = CloudComputeK8sDeleteIdempotencyLedger::default();
    let req = delete_request("req-del-missing-verifier", "idem-del-missing-verifier");

    let error = delete_cloud_compute_k8s_cluster_from_api_without_authorization_verifier(
        &catalog,
        &mut delete_ledger,
        req,
    )
    .expect_err("legacy delete entrypoint has no trusted authorization verifier");

    assert_eq!(
        error,
        CloudComputeK8sApiError::AuthorizationDenied {
            surface: CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE.to_string(),
        }
    );
    assert_eq!(error.cluster_delete_status_code(), 403);
    let planned_error = delete_cluster_without_authorization_verifier(
        &catalog,
        &mut delete_ledger,
        delete_request(
            "req-del-planned-missing-verifier",
            "idem-del-planned-missing-verifier",
        ),
    )
    .expect_err("legacy planned delete entrypoint has no trusted authorization verifier");
    assert_eq!(planned_error, error);
    assert!(delete_ledger.is_empty());
}

#[test]
fn k8s_delete_api_rejects_trusted_verifier_mismatches_before_ledger() {
    for case in [
        "forged",
        "tenant",
        "principal",
        "surface",
        "decision",
        "expired",
        "stale",
    ] {
        let (catalog, _) = catalog_with_cluster();
        let mut delete_ledger = CloudComputeK8sDeleteIdempotencyLedger::default();
        let req = delete_request(
            &format!("req-del-authz-{case}"),
            &format!("idem-del-authz-{case}"),
        );
        let mut proof = authorization_proof_for(
            "sp_compute",
            CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
            &req.authorization.decision_id,
        );
        let verifier_key = match case {
            "forged" => {
                proof.verified = false;
                None
            }
            "tenant" => {
                proof.tenant_id = "ten_other".to_string();
                None
            }
            "principal" => {
                proof.principal_id = "sp_other".to_string();
                None
            }
            "surface" => {
                proof.surface = CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE.to_string();
                None
            }
            "decision" => {
                proof.decision_id = "authz_del_other".to_string();
                Some(req.authorization.decision_id.clone())
            }
            "expired" => {
                proof.expires_at_epoch_seconds = proof.issued_at_epoch_seconds;
                None
            }
            "stale" => {
                proof.expires_at_epoch_seconds = K8S_AUTHZ_EVALUATION_EPOCH_SECONDS;
                None
            }
            _ => unreachable!("test case is exhaustive"),
        };
        let mut verifier =
            CloudComputeK8sTrustedAuthorizationVerifier::new(K8S_AUTHZ_EVALUATION_EPOCH_SECONDS);
        if let Some(decision_id) = verifier_key {
            verifier.trust_authorization_proof_for_decision(decision_id, proof);
        } else {
            verifier.trust_authorization_proof(proof);
        }

        let error = delete_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
            &catalog,
            &mut delete_ledger,
            req,
            &verifier,
        )
        .expect_err("trusted verifier mismatch is rejected before mutation");

        assert_eq!(
            error,
            CloudComputeK8sApiError::AuthorizationDenied {
                surface: CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE.to_string(),
            }
        );
        assert_eq!(error.cluster_delete_status_code(), 403);
        assert!(delete_ledger.is_empty());
    }
}

#[test]
fn k8s_delete_api_ignores_caller_supplied_authorization_proof() {
    let (catalog, _) = catalog_with_cluster();
    let mut delete_ledger = CloudComputeK8sDeleteIdempotencyLedger::default();
    let mut req = delete_request("req-del-ignore-proof", "idem-del-ignore-proof");
    req.authorization.tenant_id = "ten_forged".to_string();
    req.authorization.principal_id = "sp_forged_compute".to_string();
    req.authorization.allowed_surfaces.clear();
    req.authorization.proof = None;
    let verifier = trusted_delete_verifier_for(&req);

    let response = delete_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        &catalog,
        &mut delete_ledger,
        req,
        &verifier,
    )
    .expect("trusted verifier state, not caller proof fields, authorizes delete");

    assert_eq!(response.data.resource_id, CLUSTER_ID);
    assert_eq!(response.data.state, "draining");
    assert_eq!(delete_ledger.len(), 1);
}

#[test]
fn k8s_delete_api_rejects_tenant_mismatch_as_forbidden() {
    let (catalog, _) = catalog_with_cluster();
    let mut delete_ledger = CloudComputeK8sDeleteIdempotencyLedger::default();
    let mut req = delete_request("req-del-mismatch", "idem-del-mismatch");
    req.principal.tenant_id = "ten_other".to_string();

    let error = delete_cloud_compute_k8s_cluster_from_api(&catalog, &mut delete_ledger, req)
        .expect_err("tenant mismatch rejected");

    assert!(matches!(
        error,
        CloudComputeK8sApiError::TenantMismatch { .. }
    ));
    assert_eq!(error.cluster_delete_status_code(), 403);
    assert!(delete_ledger.is_empty());
}

#[test]
fn k8s_delete_api_rejects_unknown_cluster_as_not_found() {
    let catalog = CloudComputeCatalog::default(); // empty — no cluster
    let mut delete_ledger = CloudComputeK8sDeleteIdempotencyLedger::default();

    let error = delete_cloud_compute_k8s_cluster_from_api(
        &catalog,
        &mut delete_ledger,
        delete_request("req-del-missing", "idem-del-missing"),
    )
    .expect_err("missing cluster returns 404");

    assert!(matches!(
        error,
        CloudComputeK8sApiError::ClusterNotFound { .. }
    ));
    assert_eq!(error.cluster_delete_status_code(), 404);
    assert!(delete_ledger.is_empty());
}

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
