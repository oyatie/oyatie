#[test]
fn functions_invoke_api_rejects_path_body_drift_before_catalog_mutation() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let mut request = request(
        "req-compute-functions-drift",
        "idem-functions-drift-001",
        "fninv_drift",
    );
    request.body.function_id = "oyatie:cloud:region-home:ten_alpha:function:other".to_string();

    let error = invoke_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect_err("path/body function drift is rejected");

    assert_eq!(
        error,
        CloudComputeFunctionsApiError::FunctionIdMismatch {
            path_function_id: FUNCTION_ID.to_string(),
            body_function_id: "oyatie:cloud:region-home:ten_alpha:function:other".to_string(),
        }
    );
    assert_eq!(error.invoke_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(catalog.invocations().count(), 0);
}

#[test]
fn functions_invoke_api_separates_authentication_and_missing_authorization_verifier() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let mut missing_principal = request(
        "req-compute-functions-authn",
        "idem-functions-authn-001",
        "fninv_authn",
    );
    missing_principal.principal.principal_id.clear();

    let authn_error = invoke_with_trusted_verifier(&mut catalog, &mut ledger, missing_principal)
        .expect_err("missing authenticated principal is an authentication failure");

    assert_eq!(authn_error, CloudComputeFunctionsApiError::EmptyPrincipalId);
    assert_eq!(authn_error.invoke_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.invocations().count(), 0);

    let no_verifier = request(
        "req-compute-functions-authz-no-verifier",
        "idem-functions-authz-no-verifier-001",
        "fninv_authz_no_verifier",
    );

    let no_verifier_error =
        invoke_cloud_compute_function_from_api(&mut catalog, &mut ledger, no_verifier)
            .expect_err("authorization verifier is required at the compute boundary");

    assert_eq!(
        no_verifier_error,
        CloudComputeFunctionsApiError::AuthorizationVerifierMissing
    );
    assert_eq!(no_verifier_error.invoke_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.invocations().count(), 0);
}

#[test]
fn functions_invoke_api_rejects_trusted_authorization_binding_mismatches() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();

    let tenant_mismatch = request(
        "req-compute-functions-authz-tenant-mismatch",
        "idem-functions-authz-tenant-mismatch-001",
        "fninv_authz_tenant_mismatch",
    );
    let mut tenant_mismatch_decision = trusted_allow_for(
        &tenant_mismatch.authorization.decision_id,
        &tenant_mismatch.principal.principal_id,
    );
    tenant_mismatch_decision.tenant_id = "ten_beta".to_string();
    let tenant_mismatch_verifier =
        CloudComputeFunctionsAuthorizationVerifier::new(FUNCTIONS_AUTHZ_EVALUATION_EPOCH_SECONDS)
            .with_trusted_decision(tenant_mismatch_decision);

    let tenant_mismatch_error = invoke_cloud_compute_function_from_api_with_authorization_verifier(
        &mut catalog,
        &mut ledger,
        &tenant_mismatch_verifier,
        tenant_mismatch,
    )
    .expect_err("trusted decision tenant must match principal tenant");
    assert_eq!(
        tenant_mismatch_error,
        CloudComputeFunctionsApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: "ten_beta".to_string(),
            principal_tenant_id: "ten_alpha".to_string(),
        }
    );
    assert_eq!(tenant_mismatch_error.invoke_status_code(), 403);

    let principal_mismatch = request(
        "req-compute-functions-authz-principal-mismatch",
        "idem-functions-authz-principal-mismatch-001",
        "fninv_authz_principal_mismatch",
    );
    let principal_mismatch_verifier =
        CloudComputeFunctionsAuthorizationVerifier::new(FUNCTIONS_AUTHZ_EVALUATION_EPOCH_SECONDS)
            .with_trusted_decision(trusted_allow_for(
                &principal_mismatch.authorization.decision_id,
                "sp_other_compute",
            ));

    let principal_mismatch_error =
        invoke_cloud_compute_function_from_api_with_authorization_verifier(
            &mut catalog,
            &mut ledger,
            &principal_mismatch_verifier,
            principal_mismatch,
        )
        .expect_err("trusted decision principal must match authenticated principal");
    assert_eq!(
        principal_mismatch_error,
        CloudComputeFunctionsApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: "sp_other_compute".to_string(),
            principal_id: "sp_compute".to_string(),
        }
    );
    assert_eq!(principal_mismatch_error.invoke_status_code(), 403);

    let surface_mismatch = request(
        "req-compute-functions-authz-surface-mismatch",
        "idem-functions-authz-surface-mismatch-001",
        "fninv_authz_surface_mismatch",
    );
    let mut surface_mismatch_decision = trusted_allow_for(
        &surface_mismatch.authorization.decision_id,
        &surface_mismatch.principal.principal_id,
    );
    surface_mismatch_decision.surface = "cloud.compute.vm.create".to_string();
    let surface_mismatch_verifier =
        CloudComputeFunctionsAuthorizationVerifier::new(FUNCTIONS_AUTHZ_EVALUATION_EPOCH_SECONDS)
            .with_trusted_decision(surface_mismatch_decision);

    let surface_mismatch_error =
        invoke_cloud_compute_function_from_api_with_authorization_verifier(
            &mut catalog,
            &mut ledger,
            &surface_mismatch_verifier,
            surface_mismatch,
        )
        .expect_err("trusted decision surface must match invoke surface");
    assert_eq!(
        surface_mismatch_error,
        CloudComputeFunctionsApiError::AuthorizationDenied {
            surface: CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE.to_string(),
        }
    );
    assert_eq!(surface_mismatch_error.invoke_status_code(), 403);

    let decision_mismatch = request(
        "req-compute-functions-authz-decision-mismatch",
        "idem-functions-authz-decision-mismatch-001",
        "fninv_authz_decision_mismatch",
    );
    let decision_mismatch_verifier =
        CloudComputeFunctionsAuthorizationVerifier::new(FUNCTIONS_AUTHZ_EVALUATION_EPOCH_SECONDS)
            .with_trusted_decision(trusted_allow_for(
                "authz_decision_for_different_request",
                &decision_mismatch.principal.principal_id,
            ));

    let decision_mismatch_error =
        invoke_cloud_compute_function_from_api_with_authorization_verifier(
            &mut catalog,
            &mut ledger,
            &decision_mismatch_verifier,
            decision_mismatch,
        )
        .expect_err("request decision_id must resolve in trusted verifier state");
    assert_eq!(
        decision_mismatch_error,
        CloudComputeFunctionsApiError::AuthorizationDenied {
            surface: CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE.to_string(),
        }
    );
    assert_eq!(decision_mismatch_error.invoke_status_code(), 403);

    let denied_decision = request(
        "req-compute-functions-authz-denied-decision",
        "idem-functions-authz-denied-decision-001",
        "fninv_authz_denied_decision",
    );
    let mut denied_trusted_decision = trusted_allow_for(
        &denied_decision.authorization.decision_id,
        &denied_decision.principal.principal_id,
    );
    denied_trusted_decision.decision = CloudComputeFunctionsAuthorizationDecision::Deny;
    let denied_decision_verifier =
        CloudComputeFunctionsAuthorizationVerifier::new(FUNCTIONS_AUTHZ_EVALUATION_EPOCH_SECONDS)
            .with_trusted_decision(denied_trusted_decision);

    let denied_decision_error = invoke_cloud_compute_function_from_api_with_authorization_verifier(
        &mut catalog,
        &mut ledger,
        &denied_decision_verifier,
        denied_decision,
    )
    .expect_err("trusted denial must fail closed");
    assert_eq!(
        denied_decision_error,
        CloudComputeFunctionsApiError::AuthorizationDenied {
            surface: CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE.to_string(),
        }
    );
    assert_eq!(denied_decision_error.invoke_status_code(), 403);

    let mut expired = request(
        "req-compute-functions-authz-expired",
        "idem-functions-authz-expired-001",
        "fninv_authz_expired",
    );
    expired.body.requested_at_epoch_seconds = 1_700_099_000;
    let mut expired_decision = trusted_allow_for(
        &expired.authorization.decision_id,
        &expired.principal.principal_id,
    );
    expired_decision.valid_until_epoch_seconds = FUNCTIONS_AUTHZ_EVALUATION_EPOCH_SECONDS;
    let expired_verifier =
        CloudComputeFunctionsAuthorizationVerifier::new(FUNCTIONS_AUTHZ_EVALUATION_EPOCH_SECONDS)
            .with_trusted_decision(expired_decision);

    let expired_error = invoke_cloud_compute_function_from_api_with_authorization_verifier(
        &mut catalog,
        &mut ledger,
        &expired_verifier,
        expired,
    )
    .expect_err("expired trusted authorization proof is fail-closed");
    assert_eq!(
        expired_error,
        CloudComputeFunctionsApiError::AuthorizationDenied {
            surface: CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE.to_string(),
        }
    );
    assert_eq!(expired_error.invoke_status_code(), 403);

    assert!(ledger.is_empty());
    assert_eq!(catalog.invocations().count(), 0);
}
