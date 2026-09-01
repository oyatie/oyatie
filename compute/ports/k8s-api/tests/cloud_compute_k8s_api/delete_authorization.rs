#[tokio::test]
async fn k8s_delete_api_rejects_trusted_verifier_mismatches_before_repository() {
    for case in [
        "forged",
        "tenant",
        "principal",
        "surface",
        "decision",
        "expired",
        "stale",
    ] {
        let repository = delete_repository_with_cluster().await;
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
            &repository,
            req,
            &verifier,
        )
        .await
        .expect_err("trusted verifier mismatch is rejected before mutation");

        assert_eq!(
            error,
            CloudComputeK8sApiError::AuthorizationDenied {
                surface: CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE.to_string(),
            }
        );
        assert_eq!(error.cluster_delete_status_code(), 403);
        assert_eq!(repository.delete_operation_count(), 0);
        assert_eq!(
            stored_cluster_lifecycle(&repository),
            ("creating".to_string(), "present".to_string())
        );
    }
}

#[tokio::test]
async fn k8s_delete_api_ignores_caller_supplied_authorization_proof() {
    let repository = delete_repository_with_cluster().await;
    let mut req = delete_request("req-del-ignore-proof", "idem-del-ignore-proof");
    req.authorization.tenant_id = "ten_forged".to_string();
    req.authorization.principal_id = "sp_forged_compute".to_string();
    req.authorization.allowed_surfaces.clear();
    req.authorization.proof = None;
    let verifier = trusted_delete_verifier_for(&req);

    let response = delete_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        &repository,
        req,
        &verifier,
    )
    .await
    .expect("trusted verifier state, not caller proof fields, authorizes delete");

    assert_eq!(response.data.resource_id, CLUSTER_ID);
    assert_eq!(response.data.state, "creating");
    assert_eq!(response.data.desired_state, "deleted");
    assert_eq!(repository.delete_operation_count(), 1);
}

#[tokio::test]
async fn k8s_delete_api_rejects_tenant_mismatch_as_forbidden() {
    let repository = delete_repository_with_cluster().await;
    let mut req = delete_request("req-del-mismatch", "idem-del-mismatch");
    req.principal.tenant_id = "ten_other".to_string();

    let error = delete_cloud_compute_k8s_cluster_from_api(&repository, req)
        .await
        .expect_err("tenant mismatch rejected");

    assert!(matches!(
        error,
        CloudComputeK8sApiError::TenantMismatch { .. }
    ));
    assert_eq!(error.cluster_delete_status_code(), 403);
    assert_eq!(repository.delete_operation_count(), 0);
    assert_eq!(
        stored_cluster_lifecycle(&repository),
        ("creating".to_string(), "present".to_string())
    );
}

#[tokio::test]
async fn k8s_delete_api_rejects_unknown_cluster_as_not_found() {
    let repository = LifecycleTestRepository::default();

    let error = delete_cloud_compute_k8s_cluster_from_api(
        &repository,
        delete_request("req-del-missing", "idem-del-missing"),
    )
    .await
    .expect_err("missing cluster returns 404");

    assert!(matches!(
        error,
        CloudComputeK8sApiError::ClusterNotFound { .. }
    ));
    assert_eq!(error.cluster_delete_status_code(), 404);
    assert_eq!(repository.delete_operation_count(), 0);
}
