// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use compute_domain::{
    CloudComputeCatalog, CloudComputeError, ComputeRepo, FunctionDeploymentCreate,
    FunctionDeploymentState,
};
use compute_functions_api::{
    CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE, CloudComputeFunctionsApiAuthorization,
    CloudComputeFunctionsApiBoundaryContext, CloudComputeFunctionsApiError,
    CloudComputeFunctionsApiPrincipal, CloudComputeFunctionsAuthorizationDecision,
    CloudComputeFunctionsAuthorizationVerifier, CloudComputeFunctionsInvokeApiRequest,
    CloudComputeFunctionsInvokeApiStatus, CloudComputeFunctionsInvokeIdempotencyLedger,
    CloudComputeFunctionsInvokeRequest, CloudComputeFunctionsTrustedAuthorizationDecision, invoke,
    invoke_cloud_compute_function_from_api,
    invoke_cloud_compute_function_from_api_with_authorization_verifier,
    invoke_with_authorization_verifier,
};
use compute_resource::FunctionRuntime;
use data_boundary_kernel::DataClass;
use network_residency::ResidencyClass;

const FUNCTION_ID: &str = "oya:cloud:region-home:ten_alpha:function:image-resize";
const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const FUNCTIONS_AUTHZ_EVALUATION_EPOCH_SECONDS: u64 = 1_700_100_040;

fn boundary_for(
    request_id: &str,
    idempotency_key: &str,
) -> CloudComputeFunctionsApiBoundaryContext {
    CloudComputeFunctionsApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudComputeFunctionsApiPrincipal {
    CloudComputeFunctionsApiPrincipal {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization_for(
    principal_id: &str,
    surfaces: &[&str],
) -> CloudComputeFunctionsApiAuthorization {
    CloudComputeFunctionsApiAuthorization {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_decision_{principal_id}"),
        requested_surface: CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE.to_string(),
        valid_until_epoch_seconds: 1_700_100_060,
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
    }
}

fn trusted_allow_for(
    decision_id: &str,
    principal_id: &str,
) -> CloudComputeFunctionsTrustedAuthorizationDecision {
    CloudComputeFunctionsTrustedAuthorizationDecision {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: decision_id.to_string(),
        surface: CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE.to_string(),
        decision: CloudComputeFunctionsAuthorizationDecision::Allow,
        valid_until_epoch_seconds: 1_700_100_060,
    }
}

fn authorization_verifier_for(
    request: &CloudComputeFunctionsInvokeApiRequest,
) -> CloudComputeFunctionsAuthorizationVerifier {
    CloudComputeFunctionsAuthorizationVerifier::new(FUNCTIONS_AUTHZ_EVALUATION_EPOCH_SECONDS)
        .with_trusted_decision(trusted_allow_for(
            &request.authorization.decision_id,
            &request.principal.principal_id,
        ))
}

fn invoke_with_trusted_verifier(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeFunctionsInvokeIdempotencyLedger,
    request: CloudComputeFunctionsInvokeApiRequest,
) -> Result<
    compute_functions_api::CloudComputeFunctionsInvokeSuccessResponse,
    CloudComputeFunctionsApiError,
> {
    let authorization_verifier = authorization_verifier_for(&request);
    invoke_cloud_compute_function_from_api_with_authorization_verifier(
        catalog,
        idempotency_ledger,
        &authorization_verifier,
        request,
    )
}

fn function_create() -> FunctionDeploymentCreate {
    FunctionDeploymentCreate {
        resource_id: FUNCTION_ID.to_string(),
        tenant_id: "ten_alpha".to_string(),
        region: "region-home".to_string(),
        az: "region-home-a".to_string(),
        cell_id: "cell-region-home-a-001".to_string(),
        runtime: FunctionRuntime::Wasm,
        name: "image-resize".to_string(),
        bundle: format!("function://harbor.region-home.oya/ten_alpha/image-resize@sha256:{DIGEST}"),
        cold_start_budget_ms: 750,
        timeout_ms: 30_000,
        memory_mb: 512,
        max_concurrency: 250,
        allowed_data_classes: vec![DataClass::Public, DataClass::PiiIdentifying],
        residency: ResidencyClass::StrictHomeRegion,
        state: FunctionDeploymentState::Deploying,
        data_class: DataClass::Public,
        created_at_epoch_seconds: 1_700_100_020,
    }
}

fn seed_active_function(catalog: &mut CloudComputeCatalog) {
    let function = catalog
        .register_function(function_create())
        .expect("function registers");
    catalog
        .activate_function(&function.resource_id.value)
        .expect("function activates");
}

fn seed_deploying_function(catalog: &mut CloudComputeCatalog) {
    catalog
        .register_function(function_create())
        .expect("function registers");
}

fn body(invocation_id: &str, payload_data_class: &str) -> CloudComputeFunctionsInvokeRequest {
    CloudComputeFunctionsInvokeRequest {
        invocation_id: invocation_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        function_id: FUNCTION_ID.to_string(),
        region: "region-home".to_string(),
        payload_data_class: payload_data_class.to_string(),
        current_concurrent_invocations: 0,
        requested_at_epoch_seconds: 1_700_100_030,
    }
}

fn request(
    request_id: &str,
    idempotency_key: &str,
    invocation_id: &str,
) -> CloudComputeFunctionsInvokeApiRequest {
    CloudComputeFunctionsInvokeApiRequest {
        path_function_id: FUNCTION_ID.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_compute"),
        authorization: authorization_for("sp_compute", &[CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE]),
        body: body(invocation_id, "PII_IDENTIFYING"),
    }
}

#[test]
fn api_surface_status_contracts_are_covered() {
    assert_eq!(
        CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE,
        "cloud.compute.functions.invoke"
    );
    assert_eq!(CloudComputeFunctionsInvokeApiStatus::Accepted.code(), 202);
    assert_eq!(CloudComputeFunctionsInvokeApiStatus::BadRequest.code(), 400);
    assert_eq!(
        CloudComputeFunctionsInvokeApiStatus::Unauthorized.code(),
        401
    );
    assert_eq!(CloudComputeFunctionsInvokeApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudComputeFunctionsInvokeApiStatus::NotFound.code(), 404);
    assert_eq!(CloudComputeFunctionsInvokeApiStatus::Conflict.code(), 409);
    assert_eq!(
        CloudComputeFunctionsInvokeApiStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn functions_invoke_api_records_invocation_once_and_replays_same_idempotent_result() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let request = request(
        "req-compute-functions-invoke",
        "idem-functions-invoke-001",
        "fninv_001",
    );

    let first = invoke_with_trusted_verifier(&mut catalog, &mut ledger, request.clone())
        .expect("authorized function invoke succeeds");
    let second = invoke_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 1);
    assert_eq!(first.metadata.request_id, "req-compute-functions-invoke");
    assert_eq!(first.data.invocation_id, "fninv_001");
    assert_eq!(first.data.tenant_id, "ten_alpha");
    assert_eq!(first.data.function_id, FUNCTION_ID);
    assert_eq!(first.data.region, "region-home");
    assert_eq!(first.data.payload_data_class, "PII_IDENTIFYING");
    assert_eq!(first.data.cold_start_budget_ms, 750);
    assert_eq!(first.data.accepted_at_epoch_seconds, 1_700_100_030);
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn planned_invoke_entrypoint_fails_closed_without_verifier_and_verified_entrypoint_succeeds() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut legacy_ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let request = request(
        "req-compute-functions-invoke-alias",
        "idem-functions-invoke-alias",
        "fninv_alias",
    );

    let legacy_error = invoke(&mut catalog, &mut legacy_ledger, request.clone())
        .expect_err("legacy invoke entrypoint fails closed without compute verifier");
    assert_eq!(
        legacy_error,
        CloudComputeFunctionsApiError::AuthorizationVerifierMissing
    );
    assert_eq!(legacy_error.invoke_status_code(), 403);
    assert!(legacy_ledger.is_empty());
    assert_eq!(catalog.invocations().count(), 0);

    let authorization_verifier = authorization_verifier_for(&request);
    let mut verified_ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let response = invoke_with_authorization_verifier(
        &mut catalog,
        &mut verified_ledger,
        &authorization_verifier,
        request,
    )
    .expect("stable planned invoke entrypoint succeeds with compute verifier");

    assert_eq!(
        response.metadata.request_id,
        "req-compute-functions-invoke-alias"
    );
    assert_eq!(response.data.invocation_id, "fninv_alias");
    assert_eq!(response.data.function_id, FUNCTION_ID);
    assert_eq!(verified_ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 1);
}

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
    request.body.function_id = "oya:cloud:region-home:ten_alpha:function:other".to_string();

    let error = invoke_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect_err("path/body function drift is rejected");

    assert_eq!(
        error,
        CloudComputeFunctionsApiError::FunctionIdMismatch {
            path_function_id: FUNCTION_ID.to_string(),
            body_function_id: "oya:cloud:region-home:ten_alpha:function:other".to_string(),
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

#[test]
fn functions_invoke_api_ignores_caller_supplied_authorization_claim_fields() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let mut request = request(
        "req-compute-functions-authz-caller-fields",
        "idem-functions-authz-caller-fields-001",
        "fninv_authz_caller_fields",
    );
    let authorization_verifier = authorization_verifier_for(&request);
    request.authorization.tenant_id = "ten_beta".to_string();
    request.authorization.principal_id = "sp_forged_compute".to_string();
    request.authorization.requested_surface = "cloud.compute.vm.create".to_string();
    request.authorization.valid_until_epoch_seconds = request.body.requested_at_epoch_seconds;
    request.authorization.allowed_surfaces.clear();

    let response = invoke_cloud_compute_function_from_api_with_authorization_verifier(
        &mut catalog,
        &mut ledger,
        &authorization_verifier,
        request,
    )
    .expect("compute verifier state, not caller-supplied authorization claims, controls allow");

    assert_eq!(response.data.invocation_id, "fninv_authz_caller_fields");
    assert_eq!(response.data.function_id, FUNCTION_ID);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 1);
}

#[test]
fn functions_invoke_api_replays_with_refreshed_authz() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let request = request(
        "req-compute-functions-authz-refresh-1",
        "idem-functions-authz-refresh-001",
        "fninv_authz_refresh",
    );
    let first = invoke_with_trusted_verifier(&mut catalog, &mut ledger, request.clone())
        .expect("initial function invoke succeeds");

    let mut retry = request;
    retry.boundary.request_id = "req-compute-functions-authz-refresh-2".to_string();
    retry.authorization.decision_id = "authz_decision_sp_compute_refreshed".to_string();
    retry.authorization.allowed_surfaces = vec![
        "cloud.compute.k8s.cluster.create".to_string(),
        CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE.to_string(),
    ];
    let second = invoke_with_trusted_verifier(&mut catalog, &mut ledger, retry)
        .expect("refreshed authorization evidence does not change operation fingerprint");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 1);
}

#[test]
fn functions_invoke_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let request = request(
        "req-compute-functions-idem",
        "idem-functions-idem-001",
        "fninv_idem",
    );
    invoke_with_trusted_verifier(&mut catalog, &mut ledger, request.clone())
        .expect("initial invoke succeeds");

    let mut drifted = request;
    drifted.body.payload_data_class = "PUBLIC".to_string();
    assert_eq!(
        invoke_with_trusted_verifier(&mut catalog, &mut ledger, drifted),
        Err(CloudComputeFunctionsApiError::IdempotencyKeyReused {
            idempotency_key: "idem-functions-idem-001".to_string(),
        })
    );
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 1);
}

#[test]
fn functions_invoke_api_maps_unknown_inactive_and_duplicate_invocations() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let unknown = invoke_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request(
            "req-compute-functions-unknown",
            "idem-functions-unknown-001",
            "fninv_unknown",
        ),
    )
    .expect_err("unknown function is not found");
    assert_eq!(
        unknown,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::UnknownFunction)
    );
    assert_eq!(unknown.invoke_status_code(), 404);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 0);

    let mut inactive_catalog = CloudComputeCatalog::default();
    seed_deploying_function(&mut inactive_catalog);
    let mut inactive_ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let inactive = invoke_with_trusted_verifier(
        &mut inactive_catalog,
        &mut inactive_ledger,
        request(
            "req-compute-functions-inactive",
            "idem-functions-inactive-001",
            "fninv_inactive_api",
        ),
    )
    .expect_err("deploying functions cannot be invoked");
    assert_eq!(
        inactive,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::FunctionNotActive)
    );
    assert_eq!(inactive.invoke_status_code(), 409);
    assert_eq!(inactive_ledger.len(), 1);
    assert_eq!(inactive_catalog.invocations().count(), 0);

    let mut active_catalog = CloudComputeCatalog::default();
    seed_active_function(&mut active_catalog);
    let mut duplicate_ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    invoke_with_trusted_verifier(
        &mut active_catalog,
        &mut duplicate_ledger,
        request(
            "req-compute-functions-duplicate-a",
            "idem-functions-duplicate-a",
            "fninv_duplicate",
        ),
    )
    .expect("first invoke succeeds");
    let duplicate = invoke_with_trusted_verifier(
        &mut active_catalog,
        &mut duplicate_ledger,
        request(
            "req-compute-functions-duplicate-b",
            "idem-functions-duplicate-b",
            "fninv_duplicate",
        ),
    )
    .expect_err("same invocation id with a new idempotency key conflicts");
    assert_eq!(
        duplicate,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::DuplicateInvocation)
    );
    assert_eq!(duplicate.invoke_status_code(), 409);
    assert_eq!(duplicate_ledger.len(), 2);
    assert_eq!(active_catalog.invocations().count(), 1);
}

#[test]
fn functions_invoke_api_maps_payload_data_class_policy_without_masking() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let mut denied = request(
        "req-compute-functions-payload-denied",
        "idem-functions-payload-denied-001",
        "fninv_payload_denied",
    );
    denied.body.payload_data_class = "PHI".to_string();

    let error = invoke_with_trusted_verifier(&mut catalog, &mut ledger, denied)
        .expect_err("payload class must be allowed by deployment policy");

    assert_eq!(
        error,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::PayloadDataClassNotAllowed)
    );
    assert_eq!(error.invoke_status_code(), 403);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 0);
}

#[test]
fn functions_invoke_api_rejects_invocation_at_declared_max_concurrency() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let mut request = request(
        "req-compute-functions-concurrency",
        "idem-functions-concurrency-001",
        "fninv_concurrency",
    );
    request.body.current_concurrent_invocations = 250;

    let error = invoke_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect_err("function max_concurrency is enforced by the domain");

    assert_eq!(
        error,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::QuotaExceeded)
    );
    assert_eq!(error.invoke_status_code(), 403);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 0);
}

#[test]
fn functions_invoke_api_rejects_unknown_payload_data_class_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let mut request = request(
        "req-compute-functions-data-class",
        "idem-functions-data-class-001",
        "fninv_data_class",
    );
    request.body.payload_data_class = "NOT_A_DATA_CLASS".to_string();

    let error = invoke_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect_err("unknown payload data class label is rejected before invocation");

    assert_eq!(
        error,
        CloudComputeFunctionsApiError::InvalidPayloadDataClassLabel {
            payload_data_class: "NOT_A_DATA_CLASS".to_string(),
        }
    );
    assert_eq!(error.invoke_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(catalog.invocations().count(), 0);
}

#[test]
fn functions_invoke_idempotency_ledger_enforces_bounded_retention() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::with_max_entries(1);

    invoke_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request(
            "req-functions-bound-1",
            "idem-functions-bound-1",
            "fninv_bound_1",
        ),
    )
    .expect("first invocation succeeds");
    invoke_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request(
            "req-functions-bound-2",
            "idem-functions-bound-2",
            "fninv_bound_2",
        ),
    )
    .expect("second invocation succeeds");

    assert_eq!(ledger.len(), 1);

    let replay_error = invoke_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request(
            "req-functions-bound-replay",
            "idem-functions-bound-1",
            "fninv_bound_1",
        ),
    )
    .expect_err(
        "evicted idempotency key is no longer replayable and reaches duplicate invocation guard",
    );

    assert_eq!(
        replay_error,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::DuplicateInvocation)
    );
}
